use crate::parser::LogEntry;

pub fn format_log_line(entry: &LogEntry) -> String {
    format!(
        "{} {} {} {} {}: {}",
        entry.timestamp,
        entry.pid,
        entry.tid,
        entry.level.to_display().chars().next().unwrap_or('?'),
        entry.tag,
        entry.message
    )
}
