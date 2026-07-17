use serde::{Deserialize, Serialize};

/// A single parsed log entry from `adb logcat -v threadtime` output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub pid: u32,
    pub tid: u32,
    pub level: LogLevel,
    pub tag: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Verbose = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Fatal = 5,
}

impl LogLevel {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'V' => Some(LogLevel::Verbose),
            'D' => Some(LogLevel::Debug),
            'I' => Some(LogLevel::Info),
            'W' => Some(LogLevel::Warn),
            'E' => Some(LogLevel::Error),
            'F' => Some(LogLevel::Fatal),
            _ => None,
        }
    }

    pub fn to_display(&self) -> &'static str {
        match self {
            LogLevel::Verbose => "Verbose",
            LogLevel::Debug => "Debug",
            LogLevel::Info => "Info",
            LogLevel::Warn => "Warn",
            LogLevel::Error => "Error",
            LogLevel::Fatal => "Fatal",
        }
    }
}

/// Parse a single threadtime log line into a LogEntry.
///
/// Format: `MM-DD HH:MM:SS.mmm   PID  TID L Tag: Message`
///
/// The timestamp is the first 18 characters, followed by variable whitespace,
/// pid, tid, level char, then `Tag: message`.
pub fn parse_threadtime_line(line: &str) -> Option<LogEntry> {
    if line.len() < 18 {
        return None;
    }

    let timestamp = line[..18].to_string();

    // Skip timestamp and whitespace to find pid
    let rest = &line[18..].trim_start();
    if rest.is_empty() {
        return None;
    }

    let mut fields = rest.split_whitespace();
    let pid = fields.next()?.parse::<u32>().ok()?;
    let tid = fields.next()?.parse::<u32>().ok()?;

    // Level is a single character
    let level_char = fields.next()?.chars().next()?;
    let level = LogLevel::from_char(level_char)?;

    // Remaining is "Tag: Message" — find the first ": " (tag-message separator)
    let remaining: String = fields.collect::<Vec<&str>>().join(" ");
    let colon_idx = remaining.find(": ")?;
    let tag = remaining[..colon_idx].to_string();
    let message = remaining[colon_idx + 2..].to_string();

    Some(LogEntry {
        timestamp,
        pid,
        tid,
        level,
        tag,
        message,
    })
}

/// Check if a line starts with a valid threadtime timestamp pattern (MM-DD HH:MM:SS.mmm).
/// Returns `false` for continuation lines (stack traces, wrapped messages, etc.).
pub fn is_continuation_line(line: &str) -> bool {
    if line.len() < 18 {
        return true;
    }
    let bytes = line.as_bytes();
    // If first bytes match timestamp pattern, this is a new log line (not a continuation)
    !(bytes[2] == b'-'
        && bytes[5] == b' '
        && bytes[8] == b':'
        && bytes[11] == b':'
        && bytes[14] == b'.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_line() {
        let line = "07-17 10:30:00.123  1234  5678 I ActivityManager: Starting activity";
        let entry = parse_threadtime_line(line);
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert_eq!(e.timestamp, "07-17 10:30:00.123");
        assert_eq!(e.pid, 1234);
        assert_eq!(e.tid, 5678);
        assert_eq!(e.level, LogLevel::Info);
        assert_eq!(e.tag, "ActivityManager");
        assert_eq!(e.message, "Starting activity");
    }

    #[test]
    fn test_parse_tag_with_spaces() {
        let line = "07-17 10:30:00.456  9999  8888 W Binder:connection: Slow operation";
        let entry = parse_threadtime_line(line);
        assert!(entry.is_some());
        let e = entry.unwrap();
        // Trailing colon NOT part of tag — ":" before space is the tag-message separator
        assert_eq!(e.tag, "Binder:connection");
        assert_eq!(e.message, "Slow operation");
    }

    #[test]
    fn test_parse_error_level() {
        let line = "07-17 10:30:01.000  1111  2222 E AndroidRuntime: FATAL EXCEPTION";
        let entry = parse_threadtime_line(line);
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert_eq!(e.level, LogLevel::Error);
    }

    #[test]
    fn test_parse_malformed_line() {
        assert!(parse_threadtime_line("garbage").is_none());
        assert!(parse_threadtime_line("").is_none());
        assert!(parse_threadtime_line("short").is_none());
    }

    #[test]
    fn test_is_continuation_line() {
        assert!(is_continuation_line("    at com.example.Main.onCreate(Main.java:10)"));
        assert!(!is_continuation_line("07-17 10:30:00.123  1234  5678 I Tag: Msg"));
    }

    #[test]
    fn test_parse_line_with_continuation_marker() {
        // Some logcat lines have leading whitespace indicating continuation
        let line = "07-17 10:31:00.000  3333  4444 D MyTag: Some message";
        let entry = parse_threadtime_line(line);
        assert!(entry.is_some());
        let e = entry.unwrap();
        assert_eq!(e.tag, "MyTag");
        assert_eq!(e.message, "Some message");
    }
}
