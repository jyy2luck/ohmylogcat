use crate::adb::stream::spawn_logcat_stream;
use crate::buffer::RingBuffer;
use crate::filter::FilterCriteria;
use crate::parser::{LogEntry, LogLevel};
use crate::settings::BufferStats;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::runtime::Handle;
use tokio::sync::mpsc as tokio_mpsc;

/// Events delivered to the TUI main loop.
#[derive(Debug)]
pub enum EngineEvent {
    /// `n` new filtered rows were appended (no entry payloads — UI reads from Engine).
    RowsAppended(usize),
    /// `n` oldest matching filtered rows were dropped due to ring wrap.
    DroppedFront(usize),
    Stats(BufferStats),
    Error(String),
    /// Filtered index list was rebuilt (clear / refilter / capacity change).
    Cleared,
}

/// Manages log streaming state, buffer, and batch delivery.
///
/// Log bodies live only in `buffer`. The filtered view is a list of indices
/// into that buffer — no second full copy of entries.
pub struct Engine {
    pub buffer: Mutex<RingBuffer>,
    pub filter: Mutex<FilterCriteria>,
    /// Indices into `buffer` (0 = oldest) for entries matching the current filter.
    filtered_indices: Mutex<Vec<usize>>,
    pub is_paused: Mutex<bool>,
    pub settings_bufcap: Mutex<usize>,
    pub stop_flag: Arc<AtomicBool>,
    pub batch_epoch: AtomicU64,
    /// Count of new filtered rows since last UI notify (suppressed while paused).
    pending_appended: Mutex<usize>,
    /// Count of filtered-front drops since last UI notify.
    pending_dropped: Mutex<usize>,
    pub stats: Mutex<EngineStats>,
    event_tx: Mutex<Sender<EngineEvent>>,
    pub is_streaming: Mutex<bool>,
}

#[derive(Debug, Clone)]
pub struct EngineStats {
    pub total_received: u64,
    pub total_matched: u64,
    pub lines_per_sec: f64,
    pub last_sample_time: Instant,
    pub last_sample_count: u64,
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
        }
    }
}

impl Engine {
    pub fn new(capacity: usize) -> (Arc<Self>, Receiver<EngineEvent>) {
        let (tx, rx) = mpsc::channel();
        let engine = Arc::new(Self {
            buffer: Mutex::new(RingBuffer::new(capacity)),
            filter: Mutex::new(FilterCriteria::default()),
            filtered_indices: Mutex::new(Vec::new()),
            is_paused: Mutex::new(false),
            settings_bufcap: Mutex::new(capacity),
            stop_flag: Arc::new(AtomicBool::new(false)),
            batch_epoch: AtomicU64::new(0),
            pending_appended: Mutex::new(0),
            pending_dropped: Mutex::new(0),
            stats: Mutex::new(EngineStats::default()),
            event_tx: Mutex::new(tx),
            is_streaming: Mutex::new(false),
        });
        (engine, rx)
    }

    fn emit(engine: &Arc<Self>, event: EngineEvent) {
        let _ = engine.event_tx.lock().unwrap().send(event);
    }

    fn emit_from(&self, event: EngineEvent) {
        let _ = self.event_tx.lock().unwrap().send(event);
    }

    fn buffer_stats(engine: &Arc<Self>) -> BufferStats {
        engine.buffer_stats_self()
    }

    fn buffer_stats_self(&self) -> BufferStats {
        let buf = self.buffer.lock().unwrap();
        let stats = self.stats.lock().unwrap();
        // Single-copy estimate: ~1.2 KB/line average (strings + allocator overhead).
        BufferStats {
            count: buf.len(),
            capacity: buf.capacity(),
            lines_per_sec: stats.lines_per_sec,
            memory_estimate_mb: (buf.len() as f64 * 1.2) / 1024.0,
        }
    }

    fn ingest_entry(engine: &Arc<Self>, entry: LogEntry) {
        let paused = *engine.is_paused.lock().unwrap();
        let matches = engine.filter.lock().unwrap().matches(&entry);

        // Keep buffer + index updates in one critical section to cut lock churn
        // during the initial log flood (biggest cause of UI freezes).
        let mut dropped_match = false;
        {
            let mut buf = engine.buffer.lock().unwrap();
            let dropped = buf.push(entry);
            let new_idx = buf.len() - 1;

            let mut indices = engine.filtered_indices.lock().unwrap();
            if dropped.is_some() {
                indices.retain_mut(|idx| {
                    if *idx == 0 {
                        dropped_match = true;
                        false
                    } else {
                        *idx -= 1;
                        true
                    }
                });
            }
            if matches {
                indices.push(new_idx);
            }
        }

        {
            let mut stats = engine.stats.lock().unwrap();
            stats.total_received += 1;
            if matches {
                stats.total_matched += 1;
            }
        }

        if dropped_match {
            let mut pending = engine.pending_dropped.lock().unwrap();
            *pending = pending.saturating_add(1);
        }
        if matches && !paused {
            let mut pending = engine.pending_appended.lock().unwrap();
            *pending = pending.saturating_add(1);
        }
    }

    /// Start streaming logcat from the given device.
    pub fn start_stream(self: &Arc<Self>, handle: Handle, adb_path: String, serial: String) {
        self.stop_flag.store(false, Ordering::Relaxed);
        let epoch = self.batch_epoch.fetch_add(1, Ordering::SeqCst) + 1;

        let grow_hint = (*self.settings_bufcap.lock().unwrap()).min(16_384);
        {
            let mut buf = self.buffer.lock().unwrap();
            buf.clear();
            buf.reserve(grow_hint);
        }
        {
            let mut indices = self.filtered_indices.lock().unwrap();
            indices.clear();
            indices.reserve(grow_hint);
        }
        *self.pending_appended.lock().unwrap() = 0;
        *self.pending_dropped.lock().unwrap() = 0;
        *self.stats.lock().unwrap() = EngineStats::default();
        *self.is_paused.lock().unwrap() = false;
        *self.is_streaming.lock().unwrap() = true;

        let stop_flag = self.stop_flag.clone();
        let engine_ingest = self.clone();
        let engine_batch = self.clone();
        let engine_err = self.clone();

        let (tx, mut rx) = tokio_mpsc::unbounded_channel::<LogEntry>();

        handle.spawn(async move {
            while let Some(entry) = rx.recv().await {
                if engine_ingest.batch_epoch.load(Ordering::SeqCst) != epoch {
                    break;
                }
                Self::ingest_entry(&engine_ingest, entry);
            }
        });

        spawn_logcat_stream(
            handle.clone(),
            adb_path,
            serial,
            stop_flag,
            move |entry| {
                let _ = tx.send(entry);
            },
            move |err| {
                *engine_err.is_streaming.lock().unwrap() = false;
                Self::emit(&engine_err, EngineEvent::Error(err));
            },
        );

        handle.spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(80));
            loop {
                interval.tick().await;

                if engine_batch.batch_epoch.load(Ordering::SeqCst) != epoch {
                    break;
                }

                let appended = {
                    let mut pending = engine_batch.pending_appended.lock().unwrap();
                    std::mem::take(&mut *pending)
                };
                let dropped = {
                    let mut pending = engine_batch.pending_dropped.lock().unwrap();
                    std::mem::take(&mut *pending)
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

                if dropped > 0 {
                    Self::emit(&engine_batch, EngineEvent::DroppedFront(dropped));
                }
                if appended > 0 {
                    Self::emit(&engine_batch, EngineEvent::RowsAppended(appended));
                }

                Self::emit(
                    &engine_batch,
                    EngineEvent::Stats(Self::buffer_stats(&engine_batch)),
                );
            }
        });
    }

    pub fn stop_stream(&self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        self.batch_epoch.fetch_add(1, Ordering::SeqCst);
        *self.is_streaming.lock().unwrap() = false;
    }

    pub fn pause(&self) {
        *self.is_paused.lock().unwrap() = true;
    }

    pub fn resume(&self) {
        *self.is_paused.lock().unwrap() = false;
    }

    pub fn is_paused(&self) -> bool {
        *self.is_paused.lock().unwrap()
    }

    pub fn is_streaming(&self) -> bool {
        *self.is_streaming.lock().unwrap()
    }

    pub fn set_filter(
        &self,
        tag: Option<String>,
        message: Option<String>,
        min_level: Option<LogLevel>,
    ) {
        {
            let mut filter = self.filter.lock().unwrap();
            filter.tag_substring = tag.filter(|t| !t.is_empty());
            filter.set_message_filter(message);
            filter.min_level = min_level;
        }
        self.re_filter();
    }

    /// Re-apply filter to full buffer and rebuild the index list.
    pub fn re_filter(&self) {
        let criteria = self.filter.lock().unwrap().clone();
        let buf = self.buffer.lock().unwrap();
        let mut indices = Vec::new();
        for (i, entry) in buf.iter().enumerate() {
            if criteria.matches(entry) {
                indices.push(i);
            }
        }
        drop(buf);
        *self.filtered_indices.lock().unwrap() = indices;
        *self.pending_appended.lock().unwrap() = 0;
        *self.pending_dropped.lock().unwrap() = 0;
        Self::emit_from(self, EngineEvent::Cleared);
    }

    pub fn clear_buffer(&self) {
        self.buffer.lock().unwrap().clear_compact();
        self.filtered_indices.lock().unwrap().clear();
        *self.pending_appended.lock().unwrap() = 0;
        *self.pending_dropped.lock().unwrap() = 0;
        Self::emit_from(self, EngineEvent::Cleared);
        Self::emit_from(self, EngineEvent::Stats(self.buffer_stats_self()));
    }

    pub fn stats(&self) -> BufferStats {
        self.buffer_stats_self()
    }

    pub fn filtered_len(&self) -> usize {
        self.filtered_indices.lock().unwrap().len()
    }

    pub fn filtered_get(&self, index: usize) -> Option<LogEntry> {
        // Lock order must match ingest: buffer → indices.
        let buf = self.buffer.lock().unwrap();
        let indices = self.filtered_indices.lock().unwrap();
        let buf_idx = *indices.get(index)?;
        buf.get(buf_idx).cloned()
    }

    /// Copy a filtered index range in one lock pair (for UI paint).
    pub fn copy_filtered_range(&self, start: usize, end: usize) -> Vec<LogEntry> {
        // Lock order must match ingest: buffer → indices.
        let buf = self.buffer.lock().unwrap();
        let indices = self.filtered_indices.lock().unwrap();
        let end = end.min(indices.len());
        let start = start.min(end);
        let mut out = Vec::with_capacity(end.saturating_sub(start));
        for &buf_idx in &indices[start..end] {
            if let Some(entry) = buf.get(buf_idx) {
                out.push(entry.clone());
            }
        }
        out
    }

    pub fn set_capacity(&self, capacity: usize) {
        *self.settings_bufcap.lock().unwrap() = capacity;
        self.buffer.lock().unwrap().set_capacity(capacity);
        self.re_filter();
    }

    /// Export entries to a threadtime-like text file without building a full clone list.
    pub fn export_to_file(&self, path: &str, filtered_only: bool) -> Result<(), String> {
        use std::io::Write;

        let buf = self.buffer.lock().unwrap();
        let indices = self.filtered_indices.lock().unwrap();

        let empty = if filtered_only {
            indices.is_empty()
        } else {
            buf.is_empty()
        };
        if empty {
            return Err("No log entries to export".into());
        }

        let mut file =
            std::fs::File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;

        let write_entry = |file: &mut std::fs::File, entry: &LogEntry| -> Result<(), String> {
            let line = format!(
                "{} {:5} {:5} {} {}: {}\n",
                entry.timestamp,
                entry.pid,
                entry.tid,
                entry.level.to_display(),
                entry.tag,
                entry.message
            );
            file.write_all(line.as_bytes())
                .map_err(|e| format!("Failed to write: {}", e))
        };

        if filtered_only {
            for &idx in indices.iter() {
                if let Some(entry) = buf.get(idx) {
                    write_entry(&mut file, entry)?;
                }
            }
        } else {
            for entry in buf.iter() {
                write_entry(&mut file, entry)?;
            }
        }

        Ok(())
    }
}
