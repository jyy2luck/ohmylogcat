## MODIFIED Requirements

### Requirement: Find in log with keyboard shortcut

The system SHALL provide an in-view find bar opened by Cmd+F on macOS or Ctrl+F on Windows that searches within currently visible (filter-applied) log entries without hiding non-matching lines.

#### Scenario: Open find bar

- **WHEN** the user presses Cmd+F (macOS) or Ctrl+F (Windows)
- **THEN** a find bar appears and receives keyboard focus

#### Scenario: Case insensitive search

- **WHEN** the user enters a search query in the find bar
- **THEN** the system matches substrings case-insensitively within the formatted text of each visible log entry

#### Scenario: Highlight all matches

- **WHEN** one or more matches exist for the current query
- **THEN** all matching substrings are visually highlighted in the log list

#### Scenario: No matches

- **WHEN** the query matches no visible log entries
- **THEN** the find bar shows zero matches and no highlights are displayed

#### Scenario: Find does not filter logs

- **WHEN** the user searches with the find bar
- **THEN** all log entries remain visible and only matching substrings are highlighted
