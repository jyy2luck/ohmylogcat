use crate::parser::{LogEntry, LogLevel};

/// Filter criteria combined with AND logic.
#[derive(Debug, Clone, Default)]
pub struct FilterCriteria {
    pub tag_substring: Option<String>,
    /// Stored lowercased for case-insensitive matching.
    pub message_substring: Option<String>,
    pub min_level: Option<LogLevel>,
}

impl FilterCriteria {
    pub fn set_message_filter(&mut self, message: Option<String>) {
        self.message_substring = message
            .map(|m| m.to_lowercase())
            .filter(|m| !m.is_empty());
    }

    /// Check if an entry matches all non-None filter criteria.
    pub fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(ref tag) = self.tag_substring {
            if !tag.is_empty() && !entry.tag.contains(tag.as_str()) {
                return false;
            }
        }
        if let Some(ref msg) = self.message_substring {
            if !contains_ignore_case(&entry.message, msg) {
                return false;
            }
        }
        if let Some(ref min) = self.min_level {
            if entry.level < *min {
                return false;
            }
        }
        true
    }
}

/// Case-insensitive substring check.
/// ASCII path is allocation-free (typical for logcat); Unicode falls back to one lowercase alloc.
fn contains_ignore_case(haystack: &str, needle_lower: &str) -> bool {
    if needle_lower.is_empty() {
        return true;
    }
    if haystack.is_ascii() && needle_lower.is_ascii() {
        let hay = haystack.as_bytes();
        let needle = needle_lower.as_bytes();
        if needle.len() > hay.len() {
            return false;
        }
        return hay
            .windows(needle.len())
            .any(|window| window.eq_ignore_ascii_case(needle));
    }
    haystack.to_lowercase().contains(needle_lower)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(level: LogLevel, tag: &str, msg: &str) -> LogEntry {
        LogEntry {
            timestamp: "07-17 12:00:00.000".into(),
            pid: 1234,
            tid: 5678,
            level,
            tag: tag.into(),
            message: msg.into(),
        }
    }

    #[test]
    fn test_no_filter_accepts_all() {
        let criteria = FilterCriteria::default();
        assert!(criteria.matches(&make_entry(LogLevel::Verbose, "Any", "any")));
        assert!(criteria.matches(&make_entry(LogLevel::Error, "X", "Y")));
    }

    #[test]
    fn test_tag_filter_case_sensitive() {
        let criteria = FilterCriteria {
            tag_substring: Some("OkHttp".into()),
            ..Default::default()
        };
        assert!(criteria.matches(&make_entry(LogLevel::Debug, "OkHttp", "msg")));
        assert!(criteria.matches(&make_entry(
            LogLevel::Debug,
            "my.OkHttp.class",
            "msg"
        )));
        assert!(!criteria.matches(&make_entry(LogLevel::Debug, "okhttp", "msg")));
        assert!(!criteria.matches(&make_entry(LogLevel::Debug, "HTTP", "msg")));
    }

    #[test]
    fn test_message_filter_case_insensitive() {
        let mut criteria = FilterCriteria::default();
        criteria.set_message_filter(Some("timeout".into()));
        assert!(criteria.matches(&make_entry(
            LogLevel::Debug,
            "T",
            "request timeout occurred"
        )));
        assert!(criteria.matches(&make_entry(LogLevel::Debug, "T", "Request Timeout")));
        assert!(!criteria.matches(&make_entry(LogLevel::Debug, "T", "connection reset")));
    }

    #[test]
    fn test_message_filter_ascii_no_false_positive() {
        let mut criteria = FilterCriteria::default();
        criteria.set_message_filter(Some("abc".into()));
        assert!(criteria.matches(&make_entry(LogLevel::Debug, "T", "xxABC yy")));
        assert!(!criteria.matches(&make_entry(LogLevel::Debug, "T", "ab")));
    }

    #[test]
    fn test_min_level_filter() {
        let warn = FilterCriteria {
            min_level: Some(LogLevel::Warn),
            ..Default::default()
        };
        assert!(!warn.matches(&make_entry(LogLevel::Verbose, "T", "m")));
        assert!(!warn.matches(&make_entry(LogLevel::Debug, "T", "m")));
        assert!(!warn.matches(&make_entry(LogLevel::Info, "T", "m")));
        assert!(warn.matches(&make_entry(LogLevel::Warn, "T", "m")));
        assert!(warn.matches(&make_entry(LogLevel::Error, "T", "m")));
        assert!(warn.matches(&make_entry(LogLevel::Fatal, "T", "m")));
    }

    #[test]
    fn test_and_combination() {
        let mut criteria = FilterCriteria {
            tag_substring: Some("OkHttp".into()),
            min_level: Some(LogLevel::Warn),
            ..Default::default()
        };
        criteria.set_message_filter(Some("timeout".into()));
        assert!(criteria.matches(&make_entry(
            LogLevel::Error,
            "OkHttp",
            "request timeout"
        )));
        assert!(!criteria.matches(&make_entry(
            LogLevel::Info,
            "OkHttp",
            "request timeout"
        )));
        assert!(!criteria.matches(&make_entry(
            LogLevel::Error,
            "HTTP",
            "request timeout"
        )));
        assert!(!criteria.matches(&make_entry(
            LogLevel::Error,
            "OkHttp",
            "connection reset"
        )));
    }

    #[test]
    fn test_apply_filter() {
        let entries = vec![
            make_entry(LogLevel::Info, "TagA", "hello"),
            make_entry(LogLevel::Warn, "TagB", "world"),
            make_entry(LogLevel::Error, "TagA", "error occurred"),
        ];
        let criteria = FilterCriteria {
            tag_substring: Some("TagA".into()),
            ..Default::default()
        };
        let filtered: Vec<_> = entries
            .iter()
            .filter(|e| criteria.matches(e))
            .cloned()
            .collect();
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].message, "hello");
        assert_eq!(filtered[1].message, "error occurred");
    }
}
