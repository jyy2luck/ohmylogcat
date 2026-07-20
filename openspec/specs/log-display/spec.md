# log-display Specification

## Purpose
TBD - created by archiving change mvp-logcat-viewer. Update Purpose after archive.
## Requirements
### Requirement: Display logs in a virtualized scrollable list

The system SHALL render log entries in a virtualized list that remains responsive with buffers up to 200,000 lines.

#### Scenario: Scroll large buffer

- **WHEN** the buffer contains 100,000 or more entries and the user scrolls rapidly
- **THEN** the UI remains interactive without freezing for more than 100 ms per scroll action

### Requirement: Color-code log levels

The system SHALL display log entries with distinct colors per level aligned with common Logcat conventions: Error/Fatal prominently distinct from Warn, Info, Debug, and Verbose.

#### Scenario: Error level coloring

- **WHEN** an entry has level Error or Fatal
- **THEN** the entry is visually distinguished from Info-level entries

### Requirement: Auto-scroll to end on new logs

The system SHALL automatically scroll to the newest log entry when tail-following mode is enabled and the displayed log set changes, including incremental appends and full list refreshes from filter or device changes.

#### Scenario: Auto-scroll when tail-following is on

- **WHEN** tail-following is enabled and new logs arrive via streaming append
- **THEN** the view scrolls to show the newest entry

#### Scenario: Auto-scroll after filter refresh

- **WHEN** tail-following is enabled and the visible log list is replaced due to a filter change
- **THEN** the view scrolls to show the newest visible entry

#### Scenario: Auto-scroll after device switch

- **WHEN** tail-following is enabled and logs begin arriving for a newly selected device
- **THEN** the view scrolls to show the newest entry as logs appear

#### Scenario: No auto-scroll when tail-following is off

- **WHEN** tail-following is disabled and new logs arrive or the list refreshes
- **THEN** the scroll position is not forced to the newest entry

#### Scenario: No auto-scroll when scrolled up

- **WHEN** the user scrolls up away from the bottom while tail-following is enabled
- **THEN** tail-following is disabled and new logs do not force the scroll position to jump

#### Scenario: Find suspends auto-scroll

- **WHEN** the in-view find bar is active with one or more matches
- **THEN** tail-following scroll behavior is suspended until find is closed

### Requirement: Scroll to end control

The system SHALL provide a Scroll to End toolbar control that acts as a persistent tail-following toggle aligned with Android Studio Logcat, with visual on/off state and preference persistence across application restarts.

#### Scenario: Default tail-following on first launch

- **WHEN** the user opens the application for the first time or has no saved preference
- **THEN** tail-following is enabled by default

#### Scenario: Persist tail-following preference

- **WHEN** the user toggles Scroll to End
- **THEN** the tail-following preference is persisted and restored on the next application launch

#### Scenario: Enable tail-following from toolbar

- **WHEN** tail-following is off and the user clicks Scroll to End
- **THEN** tail-following turns on and the view jumps to the newest entry

#### Scenario: Disable tail-following from toolbar

- **WHEN** tail-following is on and the user clicks Scroll to End
- **THEN** tail-following turns off

#### Scenario: Tail-following survives list reset events

- **WHEN** tail-following is on and the user clears logs or switches devices
- **THEN** tail-following remains enabled without requiring the user to click Scroll to End again

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

