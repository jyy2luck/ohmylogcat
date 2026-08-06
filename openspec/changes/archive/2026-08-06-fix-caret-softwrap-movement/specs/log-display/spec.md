## MODIFIED Requirements

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

## ADDED Requirements

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
