use crate::adb::stream::spawn_logcat_stream;
use crate::buffer::RingBuffer;
use crate::filter::FilterCriteria;
use crate::parser::LogEntry;
use crate::BufferStats;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{AppHandle, Emitter};

/// Manages log streaming state, buffer, and batch emission.
pub struct Engine {
    pub buffer: Mutex<RingBuffer>,
    pub filter: Mutex<FilterCriteria>,
    pub is_paused: Mutex<bool>,
    pub settings_bufcap: Mutex<usize>,
    pub stop_flag: Arc<AtomicBool>,
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
            pending_batch: Mutex::new(Vec::new()),
            stats: Mutex::new(EngineStats::default()),
        })
    }

    /// Start streaming logcat from the given device.
    pub fn start_stream(self: &Arc<Self>, app_handle: AppHandle, adb_path: String, serial: String) {
        self.stop_flag.store(false, Ordering::Relaxed);
        self.buffer.lock().unwrap().clear();
        *self.pending_batch.lock().unwrap() = Vec::new();
        *self.stats.lock().unwrap() = EngineStats::default();
        *self.is_paused.lock().unwrap() = false;

        let stop_flag = self.stop_flag.clone();
        let engine = self.clone();
        let engine2 = self.clone();
        let app = app_handle.clone();

        spawn_logcat_stream(
            adb_path,
            serial,
            stop_flag,
            move |entry| {
                engine.buffer.lock().unwrap().push(entry.clone());
                let mut stats = engine.stats.lock().unwrap();
                stats.total_received += 1;

                if !*engine.is_paused.lock().unwrap() {
                    let filter = engine.filter.lock().unwrap();
                    if filter.matches(&entry) {
                        stats.total_matched += 1;
                        engine.pending_batch.lock().unwrap().push(entry);
                    }
                }
            },
            move |err| {
                let _ = app.emit("log-error", err);
            },
        );

        // Batch emitter
        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(80));
            loop {
                interval.tick().await;

                let batch: Vec<LogEntry> = {
                    let mut pending = engine2.pending_batch.lock().unwrap();
                    if pending.is_empty() {
                        continue;
                    }
                    std::mem::take(&mut *pending)
                };

                // Update lines/sec
                {
                    let mut stats = engine2.stats.lock().unwrap();
                    let now = Instant::now();
                    let elapsed = now.duration_since(stats.last_sample_time).as_secs_f64();
                    if elapsed >= 1.0 {
                        let count = stats.total_matched;
                        stats.lines_per_sec = (count - stats.last_sample_count) as f64 / elapsed;
                        stats.last_sample_time = now;
                        stats.last_sample_count = count;
                    }
                }

                let _ = app_handle.emit("log-batch", &batch);

                // Periodic stats
                let stats = engine2.stats.lock().unwrap();
                let buf = engine2.buffer.lock().unwrap();
                let _ = app_handle.emit(
                    "buffer-stats",
                    BufferStats {
                        count: buf.len(),
                        capacity: buf.capacity(),
                        lines_per_sec: stats.lines_per_sec,
                        memory_estimate_mb: (buf.len() as f64 * 0.5) / 1024.0,
                    },
                );
            }
        });
    }

    pub fn stop_stream(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }

    /// Re-apply filter to full buffer and emit snapshot.
    pub fn re_filter(&self, app_handle: &AppHandle) {
        let criteria = self.filter.lock().unwrap().clone();
        let filtered: Vec<LogEntry> = self
            .buffer
            .lock()
            .unwrap()
            .iter()
            .filter(|e| criteria.matches(e))
            .cloned()
            .collect();
        let _ = app_handle.emit("log-snapshot", &filtered);
    }

    pub fn clear_buffer(&self, app_handle: &AppHandle) {
        self.buffer.lock().unwrap().clear();
        *self.pending_batch.lock().unwrap() = Vec::new();
        let _ = app_handle.emit("log-cleared", ());
    }
}
