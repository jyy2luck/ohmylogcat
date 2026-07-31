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

The system SHALL provide a Scroll to End / Follow toolbar control that acts as a persistent tail-following toggle aligned with Android Studio Logcat, with visual on/off state and preference persistence across application restarts. Activation SHALL work via keyboard shortcut and via the toolbar control (including mouse when available).

#### Scenario: Default tail-following on first launch

- **WHEN** the user opens the application for the first time or has no saved preference
- **THEN** tail-following is enabled by default

#### Scenario: Persist tail-following preference

- **WHEN** the user toggles Scroll to End / Follow
- **THEN** the tail-following preference is persisted and restored on the next application launch

#### Scenario: Enable tail-following from toolbar

- **WHEN** tail-following is off and the user activates Scroll to End / Follow
- **THEN** tail-following turns on and the view jumps to the newest entry

#### Scenario: Disable tail-following from toolbar

- **WHEN** tail-following is on and the user activates Scroll to End / Follow
- **THEN** tail-following turns off

#### Scenario: Tail-following survives list reset events

- **WHEN** tail-following is on and the user clears logs or switches devices
- **THEN** tail-following remains enabled without requiring the user to activate Follow again

### Requirement: Soft-Wrap toggle for log lines

The system SHALL support a Soft-Wrap preference. When Soft-Wrap is off, each log entry SHALL render on a single terminal row with horizontal panning or truncation such that the full line remains reachable. When Soft-Wrap is on, long lines MAY wrap within the viewport width; exact variable-height virtualization quality is best-effort in the TUI shell.

#### Scenario: Default is no wrap

- **WHEN** the user opens the application for the first time or has no saved preference
- **THEN** Soft-Wrap is off and each log entry occupies a single row in the viewport

#### Scenario: No wrap with horizontal access

- **WHEN** Soft-Wrap is off and a log line exceeds the viewport width
- **THEN** the user can pan horizontally or otherwise reveal the clipped portion of the line

#### Scenario: Persist soft wrap preference

- **WHEN** the user toggles Soft-Wrap (when the control is available)
- **THEN** the preference is persisted across application restarts

### Requirement: Find in log with keyboard shortcut

The system SHALL provide an in-view find UI opened by `/` and, when the terminal delivers them, Cmd+F on macOS or Ctrl+F on Windows, that searches within currently visible (filter-applied) log entries without hiding non-matching lines.

#### Scenario: Open find bar

- **WHEN** the user presses `/` (or Cmd+F / Ctrl+F when delivered by the terminal)
- **THEN** a find input appears and receives keyboard focus

#### Scenario: Case insensitive search

- **WHEN** the user enters a search query in the find input
- **THEN** the system matches substrings case-insensitively within the formatted text of each visible log entry

#### Scenario: Highlight all matches

- **WHEN** one or more matches exist for the current query
- **THEN** matching substrings are visually highlighted in the log viewport (e.g. ANSI emphasis)

#### Scenario: No matches

- **WHEN** the query matches no visible log entries
- **THEN** the find UI shows zero matches and no highlights are displayed

#### Scenario: Find does not filter logs

- **WHEN** the user searches with the find UI
- **THEN** all log entries remain visible and only matching substrings are highlighted

### Requirement: Find match navigation

The system SHALL allow the user to navigate between find matches with next/previous controls or shortcuts and display the current match index and total match count.

#### Scenario: Next match

- **WHEN** the user presses Enter or the next-match shortcut while find is active with matches
- **THEN** the view scrolls to the next match and that match receives stronger highlight emphasis

#### Scenario: Previous match

- **WHEN** the user presses the previous-match shortcut while find is active with matches
- **THEN** the view scrolls to the previous match and that match receives stronger highlight emphasis

#### Scenario: Match counter

- **WHEN** matches exist for the current query
- **THEN** the find UI displays the current match position and total count (e.g. 2/15)

#### Scenario: Wrap navigation at boundaries

- **WHEN** the user navigates past the last match or before the first match
- **THEN** navigation wraps to the opposite end of the match list

### Requirement: Selection highlight distinct from find

The system SHALL render log viewport text selection with a visual style distinct from find-match highlighting so both can be recognized when find is open.

#### Scenario: Selection visible with find active

- **WHEN** find highlights are shown and the user selects log text
- **THEN** the selection highlight remains visually distinct from find match highlights

### Requirement: Log viewport caret

The system SHALL maintain a caret position in the log viewport as a character index into a formatted log entry (logical row and column). When the log viewport has focus and no text-input overlay owns the terminal cursor, the system SHALL show the terminal hardware cursor at the caret using a blinking bar style. The caret SHALL remain defined when temporarily outside the visible viewport.

#### Scenario: Caret visible with log focus

- **WHEN** focus is on the log viewport, the filtered log list is non-empty, and no modal or find text input owns the terminal cursor
- **THEN** the terminal hardware cursor is shown at the caret cell in the log viewport with a blinking bar style

#### Scenario: Caret survives leaving the viewport

- **WHEN** mouse wheel scrolling or new log arrival moves the viewport so the caret cell is off-screen
- **THEN** the caret position is unchanged and the viewport is not forced to show the caret until a keyboard caret move occurs

### Requirement: Keyboard caret movement follows viewport on key press

The system SHALL move the caret with arrow keys, Home, End, PageUp, and PageDown while the log viewport has focus. After any such keyboard caret movement, the system SHALL adjust scroll (and horizontal pan when Soft-Wrap is off) so the caret is visible. Mouse wheel scrolling and automatic tail-follow scrolling SHALL NOT by themselves move the caret or force the caret back into view.

#### Scenario: Arrow keys move caret

- **WHEN** focus is on the log viewport and the user presses Left, Right, Up, or Down without Shift
- **THEN** the caret moves by one character or one display row as appropriate, any active selection is cleared, and the viewport scrolls or pans if needed so the caret is visible

#### Scenario: Home and End are line bounds

- **WHEN** focus is on the log viewport and the user presses Home or End without Shift
- **THEN** the caret moves to the start or end of the current logical formatted log line respectively, selection is cleared, and the caret is made visible

#### Scenario: PageUp and PageDown move caret by a page

- **WHEN** focus is on the log viewport and the user presses PageUp or PageDown without Shift
- **THEN** the caret moves by approximately one viewport height in display rows, selection is cleared, and the caret is made visible

#### Scenario: Wheel does not steal caret back

- **WHEN** the caret is outside the visible viewport after mouse wheel scrolling
- **THEN** the caret stays at its logical position until the user performs a keyboard caret move

### Requirement: Keyboard range selection in log viewport

The system SHALL support extending a text selection with Shift held while moving the caret via arrow keys, Home, or End. The first Shift+move that begins a selection SHALL set the selection anchor to the caret position before that move. Clipboard contents SHALL NOT change as a side effect of creating or adjusting a selection.

#### Scenario: Shift+arrows extend selection

- **WHEN** focus is on the log viewport and the user holds Shift and presses an arrow key
- **THEN** the selection range updates between the anchor and the new caret position and nothing is written to the clipboard

#### Scenario: Shift+Home selects to line start

- **WHEN** focus is on the log viewport and the user presses Shift+Home
- **THEN** the selection covers from the caret’s prior position through the start of the current logical line (via the usual anchor/cursor normalization) and nothing is written to the clipboard

#### Scenario: Shift+End selects to line end

- **WHEN** focus is on the log viewport and the user presses Shift+End
- **THEN** the selection covers from the caret’s prior position through the end of the current logical line and nothing is written to the clipboard

### Requirement: Word and line selection extents

The system SHALL define a word as a maximal contiguous run of ASCII letters, digits, or underscore (`[A-Za-z0-9_]`). Double-click selection SHALL select the word containing the click position, or a single non-word character when the click is not on a word character. Triple-click selection SHALL select the entire logical formatted log line for the clicked entry, including when Soft-Wrap shows that entry on multiple display rows.

#### Scenario: Double-click selects identifier word

- **WHEN** the user double-clicks on a character inside an identifier such as `MyApp_1` in the log viewport
- **THEN** the selection covers exactly that contiguous `[A-Za-z0-9_]` run and the clipboard is not updated

#### Scenario: Triple-click selects logical line

- **WHEN** Soft-Wrap is on and the user triple-clicks any display row belonging to a wrapped log entry
- **THEN** the selection covers the entire formatted text of that log entry and the clipboard is not updated

### Requirement: Multi-line selection copy text

The system SHALL copy selected text as plain characters from formatted log lines, inserting newline characters between selected log entries. Copy SHALL occur only when the user explicitly invokes the copy shortcut; finishing a mouse drag or keyboard selection SHALL NOT write the clipboard.

#### Scenario: Copy spans multiple log entries

- **WHEN** the user selects text across two or more log entries and copies via the copy shortcut
- **THEN** the clipboard contains the formatted line text joined by newline characters in log order

#### Scenario: Selection alone does not copy

- **WHEN** the user completes a mouse drag selection or keyboard range selection without pressing the copy shortcut
- **THEN** the system clipboard is not modified by that selection action

### Requirement: Close find bar

The system SHALL allow the user to close the find UI and clear all find highlights.

#### Scenario: Close with Escape

- **WHEN** the find UI is open and the user presses Esc
- **THEN** the find UI closes and all find highlights are removed
