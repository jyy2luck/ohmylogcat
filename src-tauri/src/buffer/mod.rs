use crate::parser::LogEntry;

/// Fixed-capacity ring buffer for log entries.
pub struct RingBuffer {
    buffer: Vec<Option<LogEntry>>,
    capacity: usize,
    head: usize,
    count: usize,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(None);
        }
        Self {
            buffer,
            capacity,
            head: 0,
            count: 0,
        }
    }

    pub fn push(&mut self, entry: LogEntry) {
        self.buffer[self.head] = Some(entry);
        self.head = (self.head + 1) % self.capacity;
        if self.count < self.capacity {
            self.count += 1;
        }
    }

    pub fn clear(&mut self) {
        for slot in self.buffer.iter_mut() {
            *slot = None;
        }
        self.head = 0;
        self.count = 0;
    }

    /// Iterate over entries in insertion order (oldest first).
    pub fn iter(&self) -> impl Iterator<Item = &LogEntry> {
        let start = if self.count < self.capacity {
            0
        } else {
            self.head
        };
        (0..self.count).filter_map(move |i| {
            let idx = (start + i) % self.capacity;
            self.buffer[idx].as_ref()
        })
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn set_capacity(&mut self, new_cap: usize) {
        let entries: Vec<LogEntry> = self.iter().cloned().collect();
        *self = Self::new(new_cap);
        for entry in entries {
            self.push(entry);
        }
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
        buf.push(make_entry(4, "D", "4"));
        assert_eq!(buf.len(), 3);

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
    }

    #[test]
    fn test_empty_buffer() {
        let buf = RingBuffer::new(5);
        assert_eq!(buf.len(), 0);
        assert!(buf.iter().next().is_none());
    }
}
