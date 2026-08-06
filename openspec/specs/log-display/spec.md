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

The system SHALL support a Soft-Wrap preference. When Soft-Wrap is off, each log entry SHALL render on a single terminal row with horizontal panning or truncation such that the full line remains reachable. When Soft-Wrap is on, long lines SHALL wrap within the viewport width using message-column hanging indent as specified by Soft-Wrap hanging indent under message column; exact variable-height virtualization quality remains best-effort in the TUI shell.

#### Scenario: Default is no wrap

- **WHEN** the user opens the application for the first time or has no saved preference
- **THEN** Soft-Wrap is off and each log entry occupies a single row in the viewport

#### Scenario: No wrap with horizontal access

- **WHEN** Soft-Wrap is off and a log line exceeds the viewport width
- **THEN** the user can pan horizontally or otherwise reveal the clipped portion of the line

#### Scenario: Persist soft wrap preference

- **WHEN** the user toggles Soft-Wrap (when the control is available)
- **THEN** the preference is persisted across application restarts

#### Scenario: Wrap uses hanging indent

- **WHEN** Soft-Wrap is on and a log line wraps across multiple display rows
- **THEN** continuation rows are hanging-indented under the message column per Soft-Wrap hanging indent under message column

### Requirement: Fixed-width PID and TID columns

The system SHALL format each displayed log entry with PID and TID right-aligned in a fixed field width of 5 characters (space-padded), matching common `logcat -v threadtime` / Android Studio column alignment and the existing export format. Level, tag, and message SHALL begin at stable columns across entries that share the same timestamp width.

#### Scenario: Short PID and TID are padded

- **WHEN** a log entry has PID `42` and TID `7`
- **THEN** the formatted display line includes those fields as width-5 padded values (e.g. `   42` and `    7`) so the level character aligns with other rows

#### Scenario: Find and copy use padded formatting

- **WHEN** the user searches or copies text from the log viewport
- **THEN** matches and clipboard contents use the same padded formatted line text as shown in the viewport (hanging-indent display spaces on wrap continuations are not part of that logical text)

### Requirement: Soft-Wrap hanging indent under message column

When Soft-Wrap is on, the system SHALL wrap long formatted log lines so that the first display row of an entry starts at column 0, and each subsequent soft-wrapped display row of that entry is indented to the start of the **message** field (the character immediately after `{tag}: ` in the formatted line). Hanging-indent leading spaces on continuation rows are display-only: they MUST NOT be included in logical character indices, find matches, selection ranges, or copied text. When Soft-Wrap is off, this hanging indent does not apply.

#### Scenario: Continuation aligns under message

- **WHEN** Soft-Wrap is on and a formatted log line exceeds the viewport width
- **THEN** the second and later display rows for that entry begin visually under the message column, not under the timestamp

#### Scenario: Hang indent ignored when Soft-Wrap is off

- **WHEN** Soft-Wrap is off and a log line exceeds the viewport width
- **THEN** the entry remains a single display row with horizontal pan and no hanging indent

#### Scenario: Narrow viewport falls back safely

- **WHEN** Soft-Wrap is on and the computed message-column indent is greater than or equal to the viewport width
- **THEN** the system wraps without hanging indent (continuations start at column 0) rather than producing empty or unusable content rows

#### Scenario: Interaction stays on logical text

- **WHEN** Soft-Wrap is on with hanging indent and the user moves the caret, selects text, or clicks in the log viewport
- **THEN** positions map to character indices within the formatted logical line (excluding hang-indent pad spaces), and clicking within a continuation’s pad area maps to the first logical character of that wrap chunk

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

The system SHALL maintain a caret position in the log viewport as a gap index into a formatted log entry: a logical row and a column in the range `[0, line_len]`, where `col = 0` is before the first character, `col = line_len` is after the last character, and `col = k` (for `0 < k < line_len`) is between characters `k-1` and `k`. When the log viewport has focus and no text-input overlay owns the terminal cursor, the system SHALL show the terminal hardware cursor at the caret using a blinking bar style; for `col = line_len` the bar SHALL be drawn at the right edge of the last character cell (the line-end gap). The caret SHALL remain defined when temporarily outside the visible viewport.

#### Scenario: Caret visible with log focus

- **WHEN** focus is on the log viewport, the filtered log list is non-empty, and no modal or find text input owns the terminal cursor
- **THEN** the terminal hardware cursor is shown at the caret gap in the log viewport with a blinking bar style, including the line-end gap when the caret column equals the line length

#### Scenario: Caret survives leaving the viewport

- **WHEN** mouse wheel scrolling or new log arrival moves the viewport so the caret gap is off-screen
- **THEN** the caret position is unchanged and the viewport is not forced to show the caret until a keyboard caret move occurs

#### Scenario: Caret column range

- **WHEN** the caret is positioned on a non-empty formatted log line
- **THEN** the caret column is a gap index in `[0, line_len]`, never greater than the line length, and never clamped to `line_len - 1` by caret movement

### Requirement: Keyboard caret movement follows viewport on key press

The system SHALL move the caret with arrow keys, Home, End, PageUp, and PageDown while the log viewport has focus, using gap-index column semantics. After any such keyboard caret movement, the system SHALL adjust scroll (and horizontal pan when Soft-Wrap is off) so the caret is visible. Mouse wheel scrolling and automatic tail-follow scrolling SHALL NOT by themselves move the caret or force the caret back into view.

- `Home` moves the caret to the start of the current display row: when Soft-Wrap is off, that is `col = 0` of the current logical line; when Soft-Wrap is on, that is the start of the current wrap chunk (`chunk_start`), so on a continuation row the caret moves to the first logical character of that chunk rather than to `col = 0` of the logical line.
- `End` moves the caret to the end of the current display row: when Soft-Wrap is off, that is `col = line_len` of the current logical line; when Soft-Wrap is on, that is the end of the current wrap chunk — `chunk_start + chunk_len` on the final chunk (which equals `line_len`), or `chunk_start + chunk_len - 1` on a non-final chunk (the last character's left gap, since the gap after that character belongs to the next display row), so the caret does not cross display rows to the end of the whole logical line.
- `Right` advances the caret by one gap; at `col = line_len` it moves to `col = 0` of the next logical line, or stops at the end of the last line.
- `Left` moves the caret back by one gap; at `col = 0` it moves to `col = line_len` of the previous logical line, or stops at the start of the first line.

#### Scenario: Arrow keys move caret

- **WHEN** focus is on the log viewport and the user presses Left, Right, Up, or Down without Shift
- **THEN** the caret moves by one gap or one display row as appropriate, any active selection is cleared, and the viewport scrolls or pans if needed so the caret is visible

#### Scenario: End places caret at current display row end

- **WHEN** the user presses End without Shift on a non-empty line
- **THEN** the caret moves to the end gap of the current display row: with Soft-Wrap off, the line-end gap of the logical line (column equal to line length); with Soft-Wrap on, the end of the current wrap chunk, selection is cleared, and the caret is made visible

#### Scenario: Home places caret at current display row start

- **WHEN** the user presses Home without Shift
- **THEN** the caret moves to the start gap of the current display row: with Soft-Wrap off, column 0 of the current logical line; with Soft-Wrap on, the start of the current wrap chunk, selection is cleared, and the caret is made visible

#### Scenario: Soft-wrap End stays on current wrap chunk

- **WHEN** Soft-Wrap is on, a log entry wraps across multiple display rows, and the caret is on a non-final wrap chunk when the user presses End
- **THEN** the caret moves to the end of that wrap chunk and does not jump to the end of the whole logical line

#### Scenario: Soft-wrap Home on continuation row goes to chunk start

- **WHEN** Soft-Wrap is on, a log entry wraps across multiple display rows, and the caret is on a continuation wrap chunk when the user presses Home
- **THEN** the caret moves to the first logical character of that continuation chunk and does not jump to column 0 of the logical line

#### Scenario: Right at line end wraps to next line

- **WHEN** the caret is at the line-end gap of a line that is not the last line and the user presses Right
- **THEN** the caret moves to column 0 of the next logical line and is made visible

#### Scenario: Right at end of last line stops

- **WHEN** the caret is at the line-end gap of the last logical line and the user presses Right
- **THEN** the caret stays at the line-end gap and does not wrap

#### Scenario: Left at line start wraps to previous line

- **WHEN** the caret is at column 0 of a line that is not the first line and the user presses Left
- **THEN** the caret moves to the line-end gap of the previous logical line and is made visible

#### Scenario: PageUp and PageDown move caret by a page

- **WHEN** focus is on the log viewport and the user presses PageUp or PageDown without Shift
- **THEN** the caret moves by approximately one viewport height in display rows, selection is cleared, and the caret is made visible

#### Scenario: Wheel does not steal caret back

- **WHEN** the caret is outside the visible viewport after mouse wheel scrolling
- **THEN** the caret stays at its logical position until the user performs a keyboard caret move

### Requirement: Vertical caret movement tracks display column

The system SHALL keep vertical caret movement (Up/Down) visually vertical by tracking a preferred display column. The preferred display column SHALL be updated to the display column the caret actually reached whenever a vertical move could not reach the preferred display column (i.e. the target was clamped to a chunk/line boundary); otherwise the preferred display column SHALL be preserved across the move. This SHALL apply uniformly whether Soft-Wrap is on or off. Horizontal caret moves and direct placement (mouse, find) SHALL set the preferred display column to the caret's current display column.

#### Scenario: Vertical move clamps to chunk start and updates preferred column

- **WHEN** Soft-Wrap is on, the caret is on the first display row of a wrapped entry at a display column left of the continuation-row hanging indent, and the user presses Down
- **THEN** the caret moves to the start of the continuation row and the preferred display column is updated to that start's display column

#### Scenario: Vertical move back goes to directly above after clamp

- **WHEN** the caret is on a continuation display row at its chunk start after a clamped Down move and the user presses Up
- **THEN** the caret moves to the display column directly above on the previous display row, not back to the column of the pre-Down position

#### Scenario: Vertical move preserves preferred column when reached

- **WHEN** the caret is on a display row at a display column that is reachable on the adjacent display row and the user presses Up or Down
- **THEN** the caret moves to the same display column on the adjacent display row and the preferred display column is preserved

#### Scenario: Non-soft-wrap vertical move clamps at line end

- **WHEN** Soft-Wrap is off, the caret is at a column beyond the length of an adjacent logical line and the user presses Up or Down
- **THEN** the caret moves to that line's end gap and the preferred display column is updated to that line's length; moving back to a line long enough to hold the new preferred column restores vertical tracking from the reached column

### Requirement: Keyboard range selection in log viewport

The system SHALL support extending a text selection with Shift held while moving the caret via arrow keys, Home, or End. Selection endpoints are gap indices and the selection SHALL cover the half-open character range `[start, end)` between the anchor and the caret, so a selection ending at `col = line_len` includes the last character of that line. The first Shift+move that begins a selection SHALL set the selection anchor to the caret position before that move. Clipboard contents SHALL NOT change as a side effect of creating or adjusting a selection.

#### Scenario: Shift+arrows extend selection

- **WHEN** focus is on the log viewport and the user holds Shift and presses an arrow key
- **THEN** the selection range updates as a half-open range between the anchor and the new caret gap and nothing is written to the clipboard

#### Scenario: Shift+Home selects to line start

- **WHEN** the user presses Shift+Home
- **THEN** the selection covers from the caret's prior gap through the start of the current logical line (column 0) as a half-open range and nothing is written to the clipboard

#### Scenario: Shift+End selects to line end

- **WHEN** the user presses Shift+End
- **THEN** the selection covers from the caret's prior gap through the line-end gap (column equal to line length) as a half-open range, including the last character of the line, and nothing is written to the clipboard

### Requirement: Word and line selection extents

The system SHALL define a word as a maximal contiguous run of ASCII letters, digits, or underscore (`[A-Za-z0-9_]`). Double-click selection SHALL select the word containing the click position, or a single non-word character when the click is not on a word character, as a half-open range covering those character indices. Triple-click selection SHALL select the entire logical formatted log line for the clicked entry as the half-open range `[0, line_len)`, including when Soft-Wrap shows that entry on multiple display rows.

#### Scenario: Double-click selects identifier word

- **WHEN** the user double-clicks on a character inside an identifier such as `MyApp_1` in the log viewport
- **THEN** the selection covers exactly that contiguous `[A-Za-z0-9_]` run as a half-open range and the clipboard is not updated

#### Scenario: Triple-click selects logical line

- **WHEN** Soft-Wrap is on and the user triple-clicks any display row belonging to a wrapped log entry
- **THEN** the selection covers the entire formatted text of that log entry as a half-open range and the clipboard is not updated

### Requirement: Multi-line selection copy text

The system SHALL copy selected text as plain characters from formatted log lines using half-open selection endpoints, inserting newline characters between selected log entries. A selection whose end gap is `col = line_len` SHALL include the last character of that line in the copied text. Copy SHALL occur only when the user explicitly invokes the copy shortcut; finishing a mouse drag or keyboard selection SHALL NOT write the clipboard.

#### Scenario: Copy spans multiple log entries

- **WHEN** the user selects text across two or more log entries and copies via the copy shortcut
- **THEN** the clipboard contains the formatted line text joined by newline characters in log order, including the last character of each line whose end gap is part of the selection

#### Scenario: Selection alone does not copy

- **WHEN** the user completes a mouse drag selection or keyboard range selection without pressing the copy shortcut
- **THEN** the system clipboard is not modified by that selection action

### Requirement: Log viewport mouse text selection

The system SHALL allow the user to drag with the left mouse button in the log viewport to select contiguous text across one or more visible formatted log lines, using gap-index endpoints. Clicking within a character cell SHALL map to the gap on the nearer side of that character; clicking past the last visible character of a line SHALL map to the line-end gap. Selection in soft-wrap mode SHALL follow logical log entry character indices, spanning wrapped display rows within an entry. Completing a drag (mouse button up) SHALL leave the selection highlighted and SHALL NOT copy to the clipboard. Copy SHALL use Cmd+C on macOS or Ctrl+C on Windows when a non-empty selection exists.

#### Scenario: Drag select in log viewport

- **WHEN** the user presses left mouse button in the log viewport and drags
- **THEN** the selected character range is highlighted in the log viewport using gap-index endpoints

#### Scenario: Click past last character reaches line-end gap

- **WHEN** the user clicks in the cell to the right of the last character of a non-empty line (or at the right edge of the last character cell)
- **THEN** the caret is placed at the line-end gap (column equal to the line length)

#### Scenario: Clear selection outside logs

- **WHEN** the user clicks outside the log viewport
- **THEN** any active selection is cleared

#### Scenario: Mouse-up does not copy

- **WHEN** the user finishes a non-empty drag selection by releasing the left mouse button
- **THEN** the selection remains visible and the system clipboard is not modified by the mouse-up

#### Scenario: Copy selection shortcut

- **WHEN** a non-empty selection exists and the user presses Cmd+C on macOS or Ctrl+C on Windows
- **THEN** the selected plain text is copied to the system clipboard

#### Scenario: Copy does not quit

- **WHEN** the user presses Ctrl+C with no active selection
- **THEN** the application does not quit (existing Ctrl+C non-quit behavior preserved)

### Requirement: Close find bar

The system SHALL allow the user to close the find UI and clear all find highlights.

#### Scenario: Close with Escape

- **WHEN** the find UI is open and the user presses Esc
- **THEN** the find UI closes and all find highlights are removed
