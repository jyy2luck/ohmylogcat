# tui-shell Specification

## Purpose
Single-process terminal (TUI) shell for ohmylogcat using ratatui/crossterm; replaces the egui/eframe desktop shell.

## Requirements

### Requirement: Single-process terminal shell

The system SHALL run as a single-process terminal application using a TUI framework (ratatui or equivalent) with a crossterm-compatible backend, with no egui/eframe window and no embedded WebView for the main UI.

#### Scenario: Launch in terminal

- **WHEN** the user starts the application from a terminal
- **THEN** the TUI takes over the terminal (raw mode) and presents the logcat UI without opening a separate GUI window

### Requirement: Top-layer quit shortcut

The system SHALL quit the application when the user presses `q` or `Q` without Control modifier only on the top layer, defined as no modal open and the find bar not open. The system SHALL NOT treat Ctrl+C as a quit shortcut.

#### Scenario: Quit from main shell

- **WHEN** no modal is open, the find bar is closed, and the user presses `q`
- **THEN** the application exits and restores the terminal

#### Scenario: Ctrl+C does not quit

- **WHEN** the user presses Ctrl+C in any context
- **THEN** the application does not quit solely because of that key binding

#### Scenario: Quit blocked on overlay layer

- **WHEN** any modal is open or the find bar is open and the user presses `q`
- **THEN** the application does not quit

#### Scenario: q inserts in text overlay

- **WHEN** a modal with a text input field is open (filter edit, export path, or settings text field) or the find bar is open, and the user presses `q`
- **THEN** the character is inserted into the active text input and the application does not quit

#### Scenario: q no-op on non-text overlay

- **WHEN** a non-text modal is open (for example Devices or Export menu) and the user presses `q`
- **THEN** nothing happens and the application does not quit

### Requirement: Main layout hosts core logcat surfaces

The system SHALL present within the terminal: a top toolbar of primary actions, filter controls showing Tag and Message filter summaries plus Level control, a scrollable log viewport, and a status bar showing buffer usage.

#### Scenario: Initial layout visible

- **WHEN** the TUI is shown after launch
- **THEN** toolbar, filter area (including Tag and Message summaries), log viewport, and status bar are all visible and usable

### Requirement: Toolbar actions as labeled controls

The system SHALL expose primary actions (device selection entry, Pause/Resume, Clear, tail-following toggle, Export, Settings, Quit) as labeled toolbar controls that can be activated by keyboard shortcuts, and SHALL support mouse click activation when the terminal provides mouse events. The Quit control SHALL display the `q` shortcut hint.

#### Scenario: Activate pause via keyboard

- **WHEN** focus is on the log viewport and the user presses the Pause shortcut
- **THEN** streaming display pauses or resumes according to the current pause state

#### Scenario: Activate clear via toolbar affordance

- **WHEN** the user activates Clear via its toolbar control or shortcut
- **THEN** the log buffer is cleared per log-streaming requirements

#### Scenario: Quit shortcut visible on toolbar

- **WHEN** the TUI main shell is shown
- **THEN** the toolbar includes a labeled Quit control showing the `q` shortcut

### Requirement: Tag and Message filter modal editors

The system SHALL open an in-TUI modal overlay to edit Tag and Message filter strings, activated from the filter row by mouse click or by `t` / `m` from the log viewport on the top layer. Filter values SHALL apply to log filtering in real time as the user types. Pressing Esc SHALL close the modal and return focus to the log viewport without reverting filter values already applied.

#### Scenario: Open tag filter modal

- **WHEN** the user clicks the Tag summary in the filter row or presses `t` while on the top layer with log viewport focus
- **THEN** a modal overlay opens with a text input for the Tag filter substring

#### Scenario: Live tag filter apply

- **WHEN** the Tag filter modal is open and the user types characters
- **THEN** the Tag filter updates and filtered log output refreshes without requiring a separate confirm action

#### Scenario: Close tag filter modal

- **WHEN** the Tag filter modal is open and the user presses Esc
- **THEN** the modal closes, focus returns to the log viewport, and the current Tag filter value remains in effect

#### Scenario: Open message filter modal

- **WHEN** the user clicks the Message summary in the filter row or presses `m` while on the top layer with log viewport focus
- **THEN** a modal overlay opens with a text input for the Message filter substring

#### Scenario: q types in filter modal

- **WHEN** a Tag or Message filter modal is open and the user presses `q`
- **THEN** the character is inserted into the filter input and the application does not quit

### Requirement: Explicit focus model

The system SHALL maintain an explicit focus target among at least: log viewport, Level control, find input (when open), and modal dialogs. Printable keys SHALL apply to the active text input when a text-input modal or the find bar is focused. Tag and Message filter strings SHALL be edited only through filter modals, not through inline filter-row focus.

#### Scenario: Shortcuts active on log viewport

- **WHEN** focus is on the log viewport on the top layer and the user presses a log-viewport action shortcut
- **THEN** the corresponding action runs

#### Scenario: Esc closes overlay

- **WHEN** a modal is open or the find bar is open and the user presses Esc
- **THEN** the overlay closes or the find bar closes and focus returns to the log viewport according to the overlay type

### Requirement: Modal panels for settings and path prompts

The system SHALL present Settings, export path entry, and Tag/Message filter editing as in-TUI modal panels rather than native OS GUI dialogs.

#### Scenario: Open settings modal

- **WHEN** the user opens Settings from the toolbar or shortcut
- **THEN** a modal panel shows adb path and buffer configuration controls editable in the terminal

#### Scenario: Open tag filter modal from filter row

- **WHEN** the user activates Tag filter edit from the filter row
- **THEN** an in-TUI modal panel provides Tag filter text entry
