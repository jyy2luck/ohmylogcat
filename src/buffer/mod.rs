use crate::parser::LogEntry;
use std::collections::VecDeque;

/// Fixed-capacity ring buffer for log entries.
/// Grows on demand up to `capacity`; does not pre-allocate empty slots.
pub struct RingBuffer {
    entries: VecDeque<LogEntry>,
    capacity: usize,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }

    /// Push an entry. Returns the dropped oldest entry when at capacity.
    pub fn push(&mut self, entry: LogEntry) -> Option<LogEntry> {
        let dropped = if self.entries.len() >= self.capacity {
            self.entries.pop_front()
        } else {
            None
        };
        self.entries.push_back(entry);
        dropped
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        // Keep allocated capacity — clearing then immediately re-filling
        // (device switch / new stream) must not re-pay the growth tax.
    }

    /// Clear and release spare capacity (user Clear / idle compact).
    pub fn clear_compact(&mut self) {
        self.entries.clear();
        self.entries.shrink_to_fit();
    }

    /// Reserve room for upcoming growth without filling slots.
    pub fn reserve(&mut self, additional: usize) {
        self.entries.reserve(additional);
    }

    /// Iterate over entries in insertion order (oldest first).
    pub fn iter(&self) -> impl Iterator<Item = &LogEntry> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Allocated slot capacity of the underlying storage (for tests / diagnostics).
    pub fn allocated_capacity(&self) -> usize {
        self.entries.capacity()
    }

    pub fn set_capacity(&mut self, new_cap: usize) {
        let new_cap = new_cap.max(1);
        let entries: Vec<LogEntry> = self.entries.drain(..).collect();
        self.capacity = new_cap;
        self.entries = VecDeque::new();
        // Keep the newest entries if shrinking.
        let start = entries.len().saturating_sub(new_cap);
        for entry in entries.into_iter().skip(start) {
            self.entries.push_back(entry);
        }
    }

    pub fn get(&self, index: usize) -> Option<&LogEntry> {
        self.entries.get(index)
    }
}

fn make_entry(pid: u32, tag: &str, msg: &str) -> LogEntry {
    LogEntry {
        timestamp: "07-17 12:00:00.000".into(),
        pid,
        tid: pid,
        level: crate::parser::LogLevel::Info,
        tag: tag.into(),
        message: msg.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_and_iterate() {
        let mut buf = RingBuffer::new(3);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 3);

        buf.push(make_entry(1, "Tag1", "msg1"));
        buf.push(make_entry(2, "Tag2", "msg2"));
        assert_eq!(buf.len(), 2);

        let entries: Vec<_> = buf.iter().collect();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].tag, "Tag1");
        assert_eq!(entries[1].tag, "Tag2");
    }

    #[test]
    fn test_wrap_around() {
        let mut buf = RingBuffer::new(3);
        buf.push(make_entry(1, "A", "1"));
        buf.push(make_entry(2, "B", "2"));
        buf.push(make_entry(3, "C", "3"));
        assert_eq!(buf.len(), 3);

        // This should overwrite the oldest (A)
        let dropped = buf.push(make_entry(4, "D", "4"));
        assert_eq!(buf.len(), 3);
        assert_eq!(dropped.unwrap().tag, "A");

        let entries: Vec<_> = buf.iter().collect();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].tag, "B");
        assert_eq!(entries[1].tag, "C");
        assert_eq!(entries[2].tag, "D");
    }

    #[test]
    fn test_clear() {
        let mut buf = RingBuffer::new(5);
        buf.push(make_entry(1, "A", "1"));
        buf.push(make_entry(2, "B", "2"));
        buf.clear();
        assert_eq!(buf.len(), 0);
        assert!(buf.iter().next().is_none());
    }

    #[test]
    fn test_empty_stays_compact() {
        let buf = RingBuffer::new(200_000);
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 200_000);
        // Must not pre-allocate full-capacity empty slots.
        assert!(
            buf.allocated_capacity() < 1_000,
            "allocated {} slots for empty buffer",
            buf.allocated_capacity()
        );

        let mut buf = RingBuffer::new(50);
        for i in 0..10 {
            buf.push(make_entry(i, "T", "m"));
        }
        buf.clear_compact();
        assert_eq!(buf.len(), 0);
        assert!(
            buf.allocated_capacity() < 50,
            "clear_compact should shrink storage, got {}",
            buf.allocated_capacity()
        );
    }

    #[test]
    fn test_set_capacity() {
        let mut buf = RingBuffer::new(5);
        buf.push(make_entry(1, "A", "1"));
        buf.push(make_entry(2, "B", "2"));
        buf.set_capacity(10);
        assert_eq!(buf.capacity(), 10);
        assert_eq!(buf.len(), 2);
    }

    #[test]
    fn test_set_capacity_smaller() {
        let mut buf = RingBuffer::new(10);
        for i in 0..8 {
            buf.push(make_entry(i, "T", "m"));
        }
        buf.set_capacity(3);
        assert_eq!(buf.capacity(), 3);
        assert_eq!(buf.len(), 3);
        // Newest three retained
        let tags: Vec<_> = buf.iter().map(|e| e.pid).collect();
        assert_eq!(tags, vec![5, 6, 7]);
    }

    #[test]
    fn test_empty_buffer() {
        let buf = RingBuffer::new(5);
        assert_eq!(buf.len(), 0);
        assert!(buf.iter().next().is_none());
    }
}
