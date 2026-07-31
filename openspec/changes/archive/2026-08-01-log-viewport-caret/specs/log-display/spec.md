## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Multi-line selection copy text

The system SHALL copy selected text as plain characters from formatted log lines, inserting newline characters between selected log entries. Copy SHALL occur only when the user explicitly invokes the copy shortcut; finishing a mouse drag or keyboard selection SHALL NOT write the clipboard.

#### Scenario: Copy spans multiple log entries

- **WHEN** the user selects text across two or more log entries and copies via the copy shortcut
- **THEN** the clipboard contains the formatted line text joined by newline characters in log order

#### Scenario: Selection alone does not copy

- **WHEN** the user completes a mouse drag selection or keyboard range selection without pressing the copy shortcut
- **THEN** the system clipboard is not modified by that selection action
