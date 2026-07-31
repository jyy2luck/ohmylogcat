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

The system SHALL present within the terminal: a top toolbar of primary actions, filter controls showing Tag and Message filter summaries plus Level control, a scrollable log viewport, and a status bar showing buffer usage. Horizontal dividers SHALL separate the toolbar, filter row, log viewport, and status bar.

#### Scenario: Initial layout visible

- **WHEN** the TUI is shown after launch
- **THEN** toolbar, filter area (including Tag and Message summaries), log viewport, and status bar are all visible and usable, with dividers between each major zone

### Requirement: Visual section dividers

The system SHALL render full-width horizontal divider lines between the toolbar, filter row, log viewport, and status bar so the four main layout zones are visually distinct.

#### Scenario: Dividers visible on launch

- **WHEN** the TUI main shell is shown after launch
- **THEN** a horizontal divider appears between the toolbar and filter row, between the filter row and log viewport, and between the log viewport and status bar

#### Scenario: Dividers do not intercept input

- **WHEN** the user clicks or moves the mouse over a divider row
- **THEN** the divider does not define a separate hit target and mouse actions fall through to existing toolbar, filter, or log behaviors according to adjacent regions

### Requirement: Filter row shortcut labels

The system SHALL label Tag, Message, and Level controls in the filter row with inline keyboard shortcut hints matching toolbar style: `[t]Tag[value]`, `[m]Message[value]`, and `[l]Level[value]`. When Tag or Message filter is unset, the value brackets SHALL be empty (e.g. `[t]Tag[]`). Level SHALL always display the current minimum level name (default Verbose). The filter row SHALL NOT show a trailing `(t/m edit · l level · …)` hint; it MAY show `(click Tag/Message)` for mouse users.

#### Scenario: Empty filter labels

- **WHEN** no Tag or Message filter is active
- **THEN** the filter row shows `[t]Tag[]` and `[m]Message[]`

#### Scenario: Active filter labels

- **WHEN** Tag filter is `myapp` and Message filter is `error`
- **THEN** the filter row shows `[t]Tag[myapp]` and `[m]Message[error]`

#### Scenario: Level label always present

- **WHEN** the minimum log level is Warn
- **THEN** the filter row shows `[l]Level[Warn]`

#### Scenario: Shortcut hints bold

- **WHEN** the filter row is rendered
- **THEN** the `[t]`, `[m]`, and `[l]` shortcut segments are visually emphasized consistent with toolbar shortcut styling

#### Scenario: Mouse hint only

- **WHEN** the filter row is rendered on the main shell
- **THEN** the row does not include `(t/m edit · l level · …)` and may include `(click Tag/Message)`

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

The system SHALL open an in-TUI modal overlay to edit Tag and Message filter strings, activated from the filter row by mouse click or by `t` / `m` from the log viewport on the top layer. Filter values SHALL apply to log filtering in real time as the user types. Text editing in these modals SHALL support a movable insertion cursor with insert, Backspace, and Delete at the cursor per the text input insertion cursor requirement. Pressing Esc SHALL close the modal and return focus to the log viewport without reverting filter values already applied.

#### Scenario: Open tag filter modal

- **WHEN** the user clicks the Tag summary in the filter row or presses `t` while on the top layer with log viewport focus
- **THEN** a modal overlay opens with a text input for the Tag filter substring and the insertion cursor at the end of the current value

#### Scenario: Live tag filter apply

- **WHEN** the Tag filter modal is open and the user types characters
- **THEN** the Tag filter updates and filtered log output refreshes without requiring a separate confirm action

#### Scenario: Close tag filter modal

- **WHEN** the Tag filter modal is open and the user presses Esc
- **THEN** the modal closes, focus returns to the log viewport, and the current Tag filter value remains in effect

#### Scenario: Open message filter modal

- **WHEN** the user clicks the Message summary in the filter row or presses `m` while on the top layer with log viewport focus
- **THEN** a modal overlay opens with a text input for the Message filter substring and the insertion cursor at the end of the current value

#### Scenario: q types in filter modal

- **WHEN** a Tag or Message filter modal is open and the user presses `q`
- **THEN** the character is inserted at the cursor in the filter input and the application does not quit

### Requirement: Text input insertion cursor

The system SHALL maintain a visible text insertion cursor for Tag filter modal, Message filter modal, and Find bar text inputs. Each surface SHALL track an independent cursor index as a character offset into its string. When a text input surface opens, the cursor SHALL be placed at the end of the current string (index equal to string length, or zero when empty). While a text input is focused, the system SHALL show the terminal hardware cursor at the insertion point using a blinking I-beam style by default. Cursor movement alone SHALL NOT trigger filter or find recomputation; only changes to the input string SHALL.

#### Scenario: Cursor at end when opening tag filter modal

- **WHEN** the Tag filter modal opens and the current Tag filter is `myapp`
- **THEN** the insertion cursor is positioned after the last character of `myapp`

#### Scenario: Move cursor with arrow keys

- **WHEN** a Tag filter, Message filter, or Find text input is focused and the user presses Left or Right
- **THEN** the insertion cursor moves one character within bounds without changing the string

#### Scenario: Home and End keys

- **WHEN** a text input listed in this requirement is focused and the user presses Home or End
- **THEN** the insertion cursor moves to the start or end of the string respectively

#### Scenario: Insert at cursor

- **WHEN** a text input listed in this requirement is focused with the cursor not at the end and the user types a printable character
- **THEN** the character is inserted at the cursor and the cursor advances one position

#### Scenario: Backspace at cursor

- **WHEN** a text input listed in this requirement is focused with the cursor after at least one character and the user presses Backspace
- **THEN** the character immediately before the cursor is removed and the cursor moves back one position

#### Scenario: Delete at cursor

- **WHEN** a text input listed in this requirement is focused with the cursor before the end of the string and the user presses Delete
- **THEN** the character at the cursor is removed and the cursor index is unchanged

#### Scenario: Mouse click positions cursor

- **WHEN** the user clicks inside the Tag filter, Message filter, or Find input value region
- **THEN** the insertion cursor moves to the character index corresponding to the click column, clamped to the string bounds

#### Scenario: Find match navigation unchanged

- **WHEN** the Find bar is open and the user presses Enter or Shift+Enter
- **THEN** find next/previous match behavior runs and is not replaced by cursor movement

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

### Requirement: Contextual mouse cursor in log shell

The system SHALL display a text (I-beam) cursor when the mouse is over the log viewport or over an active Tag filter, Message filter, or Find bar text input value region, and a default cursor when the mouse is over other toolbar, filter, or status chrome, while mouse capture is enabled.

#### Scenario: Cursor over log viewport

- **WHEN** the mouse moves over the log viewport and no modal is open
- **THEN** the terminal cursor shape is I-beam (steady bar)

#### Scenario: Cursor over text input value region

- **WHEN** the mouse moves over the Tag filter, Message filter, or Find bar input value region
- **THEN** the terminal cursor shape is I-beam

#### Scenario: Cursor over toolbar

- **WHEN** the mouse moves over the toolbar or filter row outside text input value regions
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
