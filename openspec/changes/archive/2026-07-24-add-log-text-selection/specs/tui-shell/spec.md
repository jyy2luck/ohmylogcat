## ADDED Requirements

### Requirement: Contextual mouse cursor in log shell

The system SHALL display a text (I-beam) cursor when the mouse is over the log viewport and a default cursor when the mouse is over toolbar, filter, or status chrome, while mouse capture is enabled.

#### Scenario: Cursor over log viewport

- **WHEN** the mouse moves over the log viewport
- **THEN** the terminal cursor shape is I-beam (steady bar)

#### Scenario: Cursor over toolbar

- **WHEN** the mouse moves over the toolbar or filter row
- **THEN** the terminal cursor shape is the default pointer/arrow shape

### Requirement: Log viewport mouse text selection

The system SHALL allow the user to drag with the left mouse button in the log viewport to select contiguous text across one or more visible formatted log lines. Selection in soft-wrap mode SHALL follow logical log entry character indices, spanning wrapped display rows within an entry.

#### Scenario: Drag select in log viewport

- **WHEN** the user presses left mouse button in the log viewport and drags
- **THEN** the selected character range is highlighted in the log viewport

#### Scenario: Clear selection outside logs

- **WHEN** the user clicks outside the log viewport
- **THEN** any active selection is cleared

#### Scenario: Copy selection shortcut

- **WHEN** a non-empty selection exists and the user presses Cmd+C on macOS or Ctrl+C on Windows
- **THEN** the selected plain text is copied to the system clipboard

#### Scenario: Copy does not quit

- **WHEN** the user presses Ctrl+C with no active selection
- **THEN** the application does not quit (existing Ctrl+C non-quit behavior preserved)
