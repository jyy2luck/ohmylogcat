use crate::adb::stream::spawn_logcat_stream;
use crate::buffer::RingBuffer;
use crate::filter::FilterCriteria;
use crate::parser::LogEntry;
use crate::BufferStats;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

/// Cap pending UI batches so a slow frontend cannot stall the ingest path.
const MAX_PENDING_BATCH: usize = 5_000;

/// Manages log streaming state, buffer, and batch emission.
pub struct Engine {
    pub buffer: Mutex<RingBuffer>,
    pub filter: Mutex<FilterCriteria>,
    pub is_paused: Mutex<bool>,
    pub settings_bufcap: Mutex<usize>,
    pub stop_flag: Arc<AtomicBool>,
    /// Incremented to retire previous batch emitter / ingest tasks.
    pub batch_epoch: AtomicU64,
    /// Accumulated entries since last batch emit (already filtered).
    pub pending_batch: Mutex<Vec<LogEntry>>,
    pub stats: Mutex<EngineStats>,
}

#[derive(Debug, Clone)]
pub struct EngineStats {
    pub total_received: u64,
    pub total_matched: u64,
    pub lines_per_sec: f64,
    pub last_sample_time: Instant,
    pub last_sample_count: u64,
    pub last_emit_time: Instant,
}

impl Default for EngineStats {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            total_received: 0,
            total_matched: 0,
            lines_per_sec: 0.0,
            last_sample_time: now,
            last_sample_count: 0,
            last_emit_time: now,
        }
    }
}

impl Engine {
    pub fn new(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            buffer: Mutex::new(RingBuffer::new(capacity)),
            filter: Mutex::new(FilterCriteria::default()),
            is_paused: Mutex::new(false),
            settings_bufcap: Mutex::new(capacity),
            stop_flag: Arc::new(AtomicBool::new(false)),
            batch_epoch: AtomicU64::new(0),
            pending_batch: Mutex::new(Vec::new()),
            stats: Mutex::new(EngineStats::default()),
        })
    }

    fn emit_buffer_stats(app_handle: &AppHandle, engine: &Arc<Self>) {
        // Lock order: buffer → stats (must match ingest path).
        let payload = {
            let buf = engine.buffer.lock().unwrap();
            let stats = engine.stats.lock().unwrap();
            BufferStats {
                count: buf.len(),
                capacity: buf.capacity(),
                lines_per_sec: stats.lines_per_sec,
                memory_estimate_mb: (buf.len() as f64 * 0.5) / 1024.0,
            }
        };
        let _ = app_handle.emit("buffer-stats", payload);
    }

    fn ingest_entry(engine: &Arc<Self>, entry: LogEntry) {
        let paused = *engine.is_paused.lock().unwrap();
        let should_emit = if paused {
            false
        } else {
            engine.filter.lock().unwrap().matches(&entry)
        };

        {
            let mut buf = engine.buffer.lock().unwrap();
            buf.push(entry.clone());
            let mut stats = engine.stats.lock().unwrap();
            stats.total_received += 1;
            if should_emit {
                stats.total_matched += 1;
            }
        }

        if should_emit {
            let mut pending = engine.pending_batch.lock().unwrap();
            pending.push(entry);
            if pending.len() > MAX_PENDING_BATCH {
                let excess = pending.len() - MAX_PENDING_BATCH;
                pending.drain(0..excess);
            }
        }
    }

    /// Start streaming logcat from the given device.
    pub fn start_stream(self: &Arc<Self>, app_handle: AppHandle, adb_path: String, serial: String) {
        self.stop_flag.store(false, Ordering::Relaxed);
        let epoch = self.batch_epoch.fetch_add(1, Ordering::SeqCst) + 1;
        self.buffer.lock().unwrap().clear();
        *self.pending_batch.lock().unwrap() = Vec::new();
        *self.stats.lock().unwrap() = EngineStats::default();
        *self.is_paused.lock().unwrap() = false;

        let stop_flag = self.stop_flag.clone();
        let engine_ingest = self.clone();
        let engine_batch = self.clone();
        let app_err = app_handle.clone();

        // Decouple adb read from mutex / filter work so lock contention cannot
        // stall the stdout reader (which would fill OS pipes and freeze adb).
        let (tx, mut rx) = mpsc::unbounded_channel::<LogEntry>();

        tauri::async_runtime::spawn(async move {
            while let Some(entry) = rx.recv().await {
                if engine_ingest.batch_epoch.load(Ordering::SeqCst) != epoch {
                    break;
                }
                Self::ingest_entry(&engine_ingest, entry);
            }
        });

        spawn_logcat_stream(
            adb_path,
            serial,
            stop_flag,
            move |entry| {
                let _ = tx.send(entry);
            },
            move |err| {
                let _ = app_err.emit("log-error", err);
            },
        );

        // Batch emitter + stats heartbeat
        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(80));
            loop {
                interval.tick().await;

                if engine_batch.batch_epoch.load(Ordering::SeqCst) != epoch {
                    break;
                }

                let batch: Vec<LogEntry> = {
                    let mut pending = engine_batch.pending_batch.lock().unwrap();
                    if pending.is_empty() {
                        Vec::new()
                    } else {
                        std::mem::take(&mut *pending)
                    }
                };

                {
                    let mut stats = engine_batch.stats.lock().unwrap();
                    let now = Instant::now();
                    let elapsed = now.duration_since(stats.last_sample_time).as_secs_f64();
                    if elapsed >= 1.0 {
                        let count = stats.total_matched;
                        stats.lines_per_sec =
                            (count.saturating_sub(stats.last_sample_count)) as f64 / elapsed;
                        stats.last_sample_time = now;
                        stats.last_sample_count = count;
                    }
                }

                if !batch.is_empty() {
                    let _ = app_handle.emit("log-batch", &batch);
                }

                Self::emit_buffer_stats(&app_handle, &engine_batch);
            }
        });
    }

    pub fn stop_stream(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        self.batch_epoch.fetch_add(1, Ordering::SeqCst);
    }

    /// Re-apply filter to full buffer and emit snapshot.
    pub fn re_filter(&self, app_handle: &AppHandle) {
        const FRONTEND_DISPLAY_LIMIT: usize = 10_000;

        let criteria = self.filter.lock().unwrap().clone();
        let filtered: Vec<LogEntry> = self
            .buffer
            .lock()
            .unwrap()
            .iter()
            .filter(|e| criteria.matches(e))
            .cloned()
            .collect();

        let snapshot = if filtered.len() > FRONTEND_DISPLAY_LIMIT {
            filtered[filtered.len() - FRONTEND_DISPLAY_LIMIT..].to_vec()
        } else {
            filtered
        };

        let _ = app_handle.emit("log-snapshot", &snapshot);
    }

    pub fn clear_buffer(&self, app_handle: &AppHandle) {
        self.buffer.lock().unwrap().clear();
        *self.pending_batch.lock().unwrap() = Vec::new();
        let _ = app_handle.emit("log-cleared", ());
    }
}
