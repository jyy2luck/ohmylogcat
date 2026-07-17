use crate::parser::{LogEntry, LogLevel};

/// Filter criteria combined with AND logic.
#[derive(Debug, Clone, Default)]
pub struct FilterCriteria {
    pub tag_substring: Option<String>,
    pub message_substring: Option<String>,
    pub min_level: Option<LogLevel>,
}

impl FilterCriteria {
    /// Check if an entry matches all non-None filter criteria.
    pub fn matches(&self, entry: &LogEntry) -> bool {
        if let Some(ref tag) = self.tag_substring {
            if !tag.is_empty() && !entry.tag.contains(tag.as_str()) {
                return false;
            }
        }
        if let Some(ref msg) = self.message_substring {
            if !msg.is_empty()
                && !entry
                    .message
                    .to_lowercase()
                    .contains(&msg.to_lowercase())
            {
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

/// Apply filter to an iterator of entries, returning matching entries.
pub fn apply_filter<'a>(
    entries: impl Iterator<Item = &'a LogEntry>,
    criteria: &FilterCriteria,
) -> Vec<LogEntry> {
    entries
        .filter(|e| criteria.matches(e))
        .cloned()
        .collect()
}

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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(criteria.matches(&make_entry(LogLevel::Debug, "my.OkHttp.class", "msg")));
        assert!(!criteria.matches(&make_entry(LogLevel::Debug, "okhttp", "msg")));
        assert!(!criteria.matches(&make_entry(LogLevel::Debug, "HTTP", "msg")));
    }

    #[test]
    fn test_message_filter_case_insensitive() {
        let criteria = FilterCriteria {
            message_substring: Some("timeout".into()),
            ..Default::default()
        };
        assert!(criteria.matches(&make_entry(LogLevel::Debug, "T", "request timeout occurred")));
        assert!(criteria.matches(&make_entry(LogLevel::Debug, "T", "Request Timeout")));
        assert!(!criteria.matches(&make_entry(LogLevel::Debug, "T", "connection reset")));
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
        let criteria = FilterCriteria {
            tag_substring: Some("OkHttp".into()),
            message_substring: Some("timeout".into()),
            min_level: Some(LogLevel::Warn),
        };
        // All match
        assert!(criteria.matches(&make_entry(LogLevel::Error, "OkHttp", "request timeout")));
        // Wrong level
        assert!(!criteria.matches(&make_entry(LogLevel::Info, "OkHttp", "request timeout")));
        // Wrong tag
        assert!(!criteria.matches(&make_entry(LogLevel::Error, "HTTP", "request timeout")));
        // Wrong message
        assert!(!criteria.matches(&make_entry(LogLevel::Error, "OkHttp", "connection reset")));
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
        let filtered = apply_filter(entries.iter(), &criteria);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].message, "hello");
        assert_eq!(filtered[1].message, "error occurred");
    }
}
