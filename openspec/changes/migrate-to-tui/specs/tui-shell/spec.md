## ADDED Requirements

### Requirement: Single-process terminal shell

The system SHALL run as a single-process terminal application using a TUI framework (ratatui or equivalent) with a crossterm-compatible backend, with no egui/eframe window and no embedded WebView for the main UI.

#### Scenario: Launch in terminal

- **WHEN** the user starts the application from a terminal
- **THEN** the TUI takes over the terminal (raw mode) and presents the logcat UI without opening a separate GUI window

### Requirement: Main layout hosts core logcat surfaces

The system SHALL present within the terminal: a top toolbar of primary actions, filter controls including Tag and Message text fields, a scrollable log viewport, and a status bar showing buffer usage.

#### Scenario: Initial layout visible

- **WHEN** the TUI is shown after launch
- **THEN** toolbar, filter area (including Tag and Message inputs), log viewport, and status bar are all visible and usable

### Requirement: Toolbar actions as labeled controls

The system SHALL expose primary actions (device selection entry, Pause/Resume, Clear, tail-following toggle, Export, Settings) as labeled toolbar controls that can be activated by keyboard shortcuts, and SHALL support mouse click activation when the terminal provides mouse events.

#### Scenario: Activate pause via keyboard

- **WHEN** focus is on the log viewport and the user presses the Pause shortcut
- **THEN** streaming display pauses or resumes according to the current pause state

#### Scenario: Activate clear via toolbar affordance

- **WHEN** the user activates Clear via its toolbar control or shortcut
- **THEN** the log buffer is cleared per log-streaming requirements

### Requirement: Focusable Tag and Message filter inputs

The system SHALL provide focusable text input fields for Tag and Message filters in the filter area so the user can type filter substrings without leaving the TUI.

#### Scenario: Edit tag filter

- **WHEN** the user moves focus to the Tag field and types characters
- **THEN** those characters update the Tag filter input and do not trigger log-viewport action shortcuts

#### Scenario: Leave filter input

- **WHEN** the Tag or Message field is focused and the user presses Esc or Tabs back to the log viewport
- **THEN** focus returns to the log viewport and viewport shortcuts are active again

### Requirement: Explicit focus model

The system SHALL maintain an explicit focus target among at least: log viewport, Tag field, Message field, Level control, find input (when open), and modal dialogs; printable keys SHALL apply to the focused text field when a text field is focused.

#### Scenario: Shortcuts ignored while typing message filter

- **WHEN** the Message field is focused and the user types the letter that would otherwise clear logs
- **THEN** the letter is inserted into the Message field and the buffer is not cleared

### Requirement: Modal panels for settings and path prompts

The system SHALL present Settings and export path entry as in-TUI modal panels rather than native OS GUI dialogs.

#### Scenario: Open settings modal

- **WHEN** the user opens Settings from the toolbar or shortcut
- **THEN** a modal panel shows adb path and buffer configuration controls editable in the terminal
