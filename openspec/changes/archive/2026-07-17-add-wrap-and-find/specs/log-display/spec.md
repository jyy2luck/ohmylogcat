## ADDED Requirements

### Requirement: Soft-Wrap toggle for log lines

The system SHALL provide a Soft-Wrap toolbar control that toggles between single-line (no wrap) and automatic line wrapping for log entries, aligned with Android Studio Logcat Soft-Wrap behavior.

#### Scenario: Default is no wrap

- **WHEN** the user opens the application for the first time or has no saved preference
- **THEN** Soft-Wrap is off and each log entry renders on a single line without ellipsis truncation

#### Scenario: No wrap with horizontal scroll

- **WHEN** Soft-Wrap is off and a log line exceeds the viewport width
- **THEN** the user can scroll horizontally to view the full line content

#### Scenario: Enable soft wrap

- **WHEN** the user enables Soft-Wrap from the toolbar
- **THEN** long log lines wrap within the viewport using automatic line breaking

#### Scenario: Persist soft wrap preference

- **WHEN** the user toggles Soft-Wrap
- **THEN** the preference is persisted across application restarts

### Requirement: Find in log with keyboard shortcut

The system SHALL provide an in-view find bar opened by Cmd+F on macOS or Ctrl+F on Windows that searches within currently visible (filter-applied) log entries without hiding non-matching lines.

#### Scenario: Open find bar

- **WHEN** the user presses Cmd+F (macOS) or Ctrl+F (Windows)
- **THEN** a find bar appears, receives keyboard focus, and the default browser find behavior is suppressed

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

### Requirement: Find match navigation

The system SHALL allow the user to navigate between find matches with next/previous controls and display the current match index and total match count.

#### Scenario: Next match

- **WHEN** the user presses Enter or clicks the next control in the find bar
- **THEN** the view scrolls to the next match and that match receives stronger highlight emphasis

#### Scenario: Previous match

- **WHEN** the user presses Shift+Enter or clicks the previous control in the find bar
- **THEN** the view scrolls to the previous match and that match receives stronger highlight emphasis

#### Scenario: Match counter

- **WHEN** matches exist for the current query
- **THEN** the find bar displays the current match position and total count (e.g. 2/15)

#### Scenario: Wrap navigation at boundaries

- **WHEN** the user navigates past the last match or before the first match
- **THEN** navigation wraps to the opposite end of the match list

### Requirement: Close find bar

The system SHALL allow the user to close the find bar and clear all find highlights.

#### Scenario: Close with Escape

- **WHEN** the find bar is open and the user presses Esc
- **THEN** the find bar closes and all find highlights are removed

#### Scenario: Close with dismiss control

- **WHEN** the user clicks the close control on the find bar
- **THEN** the find bar closes and all find highlights are removed
