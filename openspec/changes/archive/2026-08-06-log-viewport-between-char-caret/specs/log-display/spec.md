## MODIFIED Requirements

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

- `Home` moves the caret to `col = 0` of the current logical line.
- `End` moves the caret to `col = line_len` of the current logical line (the line-end gap, after the last character).
- `Right` advances the caret by one gap; at `col = line_len` it moves to `col = 0` of the next logical line, or stops at the end of the last line.
- `Left` moves the caret back by one gap; at `col = 0` it moves to `col = line_len` of the previous logical line, or stops at the start of the first line.

#### Scenario: Arrow keys move caret

- **WHEN** focus is on the log viewport and the user presses Left, Right, Up, or Down without Shift
- **THEN** the caret moves by one gap or one display row as appropriate, any active selection is cleared, and the viewport scrolls or pans if needed so the caret is visible

#### Scenario: End places caret after the last character

- **WHEN** focus is on the log viewport and the user presses End without Shift on a non-empty line
- **THEN** the caret moves to the line-end gap (column equal to the line length), the bar is drawn at the right edge of the last character, selection is cleared, and the caret is made visible

#### Scenario: Home places caret before the first character

- **WHEN** focus is on the log viewport and the user presses Home without Shift
- **THEN** the caret moves to column 0 of the current logical line, selection is cleared, and the caret is made visible

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

### Requirement: Keyboard range selection in log viewport

The system SHALL support extending a text selection with Shift held while moving the caret via arrow keys, Home, or End. Selection endpoints are gap indices and the selection SHALL cover the half-open character range `[start, end)` between the anchor and the caret, so a selection ending at `col = line_len` includes the last character of that line. The first Shift+move that begins a selection SHALL set the selection anchor to the caret position before that move. Clipboard contents SHALL NOT change as a side effect of creating or adjusting a selection.

#### Scenario: Shift+arrows extend selection

- **WHEN** focus is on the log viewport and the user holds Shift and presses an arrow key
- **THEN** the selection range updates as a half-open range between the anchor and the new caret gap and nothing is written to the clipboard

#### Scenario: Shift+Home selects to line start

- **WHEN** focus is on the log viewport and the user presses Shift+Home
- **THEN** the selection covers from the caret's prior gap through the start of the current logical line (column 0) as a half-open range and nothing is written to the clipboard

#### Scenario: Shift+End selects to line end

- **WHEN** focus is on the log viewport and the user presses Shift+End
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
