use crate::parser::LogEntry;

pub fn format_log_line(entry: &LogEntry) -> String {
    format!(
        "{} {:5} {:5} {} {}: {}",
        entry.timestamp,
        entry.pid,
        entry.tid,
        entry.level.to_display().chars().next().unwrap_or('?'),
        entry.tag,
        entry.message
    )
}

/// Character index where the message field begins (after `{tag}: `).
pub fn message_column_indent(entry: &LogEntry) -> usize {
    format!(
        "{} {:5} {:5} {} {}: ",
        entry.timestamp,
        entry.pid,
        entry.tid,
        entry.level.to_display().chars().next().unwrap_or('?'),
        entry.tag,
    )
    .chars()
    .count()
}

/// Message-column indent derived from a formatted line (first `: ` separator).
pub fn message_column_indent_line(line: &str) -> usize {
    line.find(": ").map(|i| i + 2).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::LogLevel;

    fn sample_entry() -> LogEntry {
        LogEntry {
            timestamp: "07-17 10:30:00.123".into(),
            pid: 42,
            tid: 7,
            level: LogLevel::Info,
            tag: "MyApp".into(),
            message: "hello".into(),
        }
    }

    #[test]
    fn pads_short_pid_tid() {
        let line = format_log_line(&sample_entry());
        assert!(line.contains("   42"));
        assert!(line.contains("    7"));
    }

    #[test]
    fn message_column_indent_matches_prefix() {
        let entry = sample_entry();
        let line = format_log_line(&entry);
        let indent = message_column_indent(&entry);
        assert_eq!(indent, message_column_indent_line(&line));
        let chars: Vec<char> = line.chars().collect();
        let msg: String = chars[indent..].iter().collect();
        assert_eq!(msg, "hello");
    }
}
