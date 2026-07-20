use crate::parser::{LogEntry, LogLevel};
use ratatui::style::Color;

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

pub fn level_color(level: LogLevel) -> Color {
    match level {
        LogLevel::Verbose => Color::DarkGray,
        LogLevel::Debug => Color::Cyan,
        LogLevel::Info => Color::White,
        LogLevel::Warn => Color::Yellow,
        LogLevel::Error | LogLevel::Fatal => Color::Red,
    }
}
