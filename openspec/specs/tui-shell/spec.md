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

### Requirement: Shell viewport inset

The system SHALL render the main shell (toolbar, filter row, section dividers, log viewport, and status bar) inside a uniform inset from the terminal edges rather than flush against the terminal border. When the terminal is large enough, the inset SHALL be at least one column on the left and right and one row on the top and bottom. When the terminal is too small to preserve a usable log viewport, the system MAY reduce or disable the inset. Modal overlays SHALL continue to use the full terminal area for centering and sizing. Mouse hit regions and log viewport dimensions SHALL match the inset content area so text selection and click targets remain accurate.

#### Scenario: Inset visible on launch

- **WHEN** the TUI main shell is shown in a terminal with sufficient width and height
- **THEN** the toolbar, filter row, log viewport, and status bar are rendered inset from all four terminal edges with visible empty gutter space around the shell

#### Scenario: Inset disabled on very small terminal

- **WHEN** the terminal width or height falls below the minimum threshold for a usable inset
- **THEN** the main shell MAY render edge-to-edge without inset while remaining usable

#### Scenario: Log selection respects inset log area

- **WHEN** the user drags to select text in the log viewport
- **THEN** selection mapping uses the inset log viewport bounds and does not treat gutter cells outside the inset as log content

#### Scenario: Modals ignore shell inset

- **WHEN** a modal overlay is open
- **THEN** the modal is centered and sized relative to the full terminal area, not the inset shell content area

#### Scenario: Mouse in gutter does not start log selection

- **WHEN** the user clicks in the gutter outside the inset log viewport
- **THEN** log text selection does not start and existing chrome click or clear-selection behavior applies as for clicks outside the log viewport

### Requirement: Main layout hosts core logcat surfaces

The system SHALL present within the terminal: a top toolbar of primary actions, filter controls showing Tag and Message filter summaries plus Level control, a scrollable log viewport, and a status bar showing buffer usage. These surfaces SHALL be laid out inside the shell viewport inset when inset is active. Horizontal dividers SHALL separate the toolbar, filter row, log viewport, and status bar.

#### Scenario: Initial layout visible

- **WHEN** the TUI is shown after launch
- **THEN** toolbar, filter area (including Tag and Message summaries), log viewport, and status bar are all visible and usable inside the shell content area, with dividers between each major zone

### Requirement: Visual section dividers

The system SHALL render horizontal divider lines between the toolbar, filter row, log viewport, and status bar so the four main layout zones are visually distinct. Divider lines SHALL span the width of the shell content area (inset width when inset is active), not the full terminal width when inset is active.

#### Scenario: Dividers visible on launch

- **WHEN** the TUI main shell is shown after launch with shell inset active
- **THEN** a horizontal divider appears between the toolbar and filter row, between the filter row and log viewport, and between the log viewport and status bar, each aligned with the inset content width

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

The system SHALL open an in-TUI modal overlay to edit Tag and Message filter strings, activated from the filter row by mouse click or by `t` / `m` from the log viewport on the top layer. Filter values SHALL apply to log filtering in real time as the user types. Text editing in these modals SHALL support a movable insertion cursor with insert, Backspace, and Delete at the cursor per the text input insertion cursor requirement. Pressing Enter SHALL close the modal and return focus to the log viewport without reverting filter values already applied. Pressing Esc SHALL use progressive dismiss: when the filter input is non-empty, Esc SHALL clear the input and the live filter SHALL update accordingly; when the input is already empty, Esc SHALL close the modal and return focus to the log viewport.

#### Scenario: Open tag filter modal

- **WHEN** the user clicks the Tag summary in the filter row or presses `t` while on the top layer with log viewport focus
- **THEN** a modal overlay opens with a text input for the Tag filter substring and the insertion cursor at the end of the current value

#### Scenario: Live tag filter apply

- **WHEN** the Tag filter modal is open and the user types characters
- **THEN** the Tag filter updates and filtered log output refreshes without requiring a separate confirm action

#### Scenario: Enter closes tag filter modal

- **WHEN** the Tag filter modal is open and the user presses Enter
- **THEN** the modal closes, focus returns to the log viewport, and the current Tag filter value remains in effect

#### Scenario: Esc clears non-empty tag filter input

- **WHEN** the Tag filter modal is open, the input is non-empty, and the user presses Esc
- **THEN** the input is cleared, the Tag filter is removed from active filtering in real time, and the modal remains open

#### Scenario: Esc closes empty tag filter modal

- **WHEN** the Tag filter modal is open, the input is empty, and the user presses Esc
- **THEN** the modal closes and focus returns to the log viewport

#### Scenario: Open message filter modal

- **WHEN** the user clicks the Message summary in the filter row or presses `m` while on the top layer with log viewport focus
- **THEN** a modal overlay opens with a text input for the Message filter substring and the insertion cursor at the end of the current value

#### Scenario: Esc clears non-empty message filter input

- **WHEN** the Message filter modal is open, the input is non-empty, and the user presses Esc
- **THEN** the input is cleared, the Message filter is removed from active filtering in real time, and the modal remains open

#### Scenario: Esc closes empty message filter modal

- **WHEN** the Message filter modal is open, the input is empty, and the user presses Esc
- **THEN** the modal closes and focus returns to the log viewport

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
- **THEN** the overlay closes or the find bar closes and focus returns to the log viewport according to the overlay type, except Tag and Message filter modals where Esc clears non-empty input before closing on a subsequent Esc when empty

### Requirement: Settings modal keyboard navigation

The Settings modal SHALL use vertical arrow keys to move focus among visible settings rows and horizontal arrow keys to adjust cycle-type fields (buffer preset, theme, and language). Adjustments and text edits SHALL apply immediately to runtime state and persist to settings storage without requiring Enter. The modal SHALL display a help line at the top documenting move, adjust, text entry, and dismiss bindings. Focus SHALL always rest on a visible row; the Custom capacity row SHALL be included in navigation only when the buffer preset is Custom. Pressing Enter or Esc SHALL close the modal and return focus to the log viewport without an additional save or cancel step.

#### Scenario: Move focus with vertical keys

- **WHEN** the Settings modal is open and the user presses Up or Down
- **THEN** the focus cursor moves to the previous or next visible settings row and the `>` marker follows the focused row

#### Scenario: Adjust preset with horizontal keys

- **WHEN** the Settings modal is open, buffer preset is focused, and the user presses Left or Right
- **THEN** the buffer preset cycles backward or forward, buffer capacity updates immediately, and the new value is persisted

#### Scenario: Adjust theme with horizontal keys

- **WHEN** the Settings modal is open, theme is focused, and the user presses Left or Right
- **THEN** the theme preference cycles backward or forward, the active theme updates immediately, and the new value is persisted

#### Scenario: Adjust language with horizontal keys

- **WHEN** the Settings modal is open, language is focused, and the user presses Left or Right
- **THEN** the language preference cycles backward or forward, UI chrome updates immediately, and the new value is persisted

#### Scenario: Text fields use direct typing

- **WHEN** the Settings modal is open, ADB path or Custom capacity is focused, and the user types printable characters or presses Backspace
- **THEN** the focused text field is edited append-only, Left/Right do not change the field value, and the updated value is persisted after the edit

#### Scenario: Custom row hidden from navigation

- **WHEN** the Settings modal is open and the buffer preset is not Custom
- **THEN** Up/Down navigation skips the Custom capacity row and focus never rests on a row that is not rendered

#### Scenario: Focus re-anchors when Custom row hides

- **WHEN** the Settings modal is open, Custom capacity is focused, and the user cycles preset away from Custom
- **THEN** focus moves to an adjacent visible row (preset, theme, or language) and the Custom row is no longer focused

#### Scenario: Help line documents controls

- **WHEN** the Settings modal is rendered
- **THEN** the first content line documents vertical move, horizontal adjust with immediate apply, text entry, and Enter/Esc dismiss

#### Scenario: Enter or Esc dismisses settings modal

- **WHEN** the Settings modal is open and the user presses Enter or Esc
- **THEN** the modal closes and focus returns to the log viewport without reverting settings already applied and persisted

### Requirement: Modal panels for settings and path prompts

The system SHALL present Settings, export path entry, and Tag/Message filter editing as in-TUI modal panels rather than native OS GUI dialogs. The Settings modal SHALL follow the Settings modal keyboard navigation requirement.

#### Scenario: Open settings modal

- **WHEN** the user opens Settings from the toolbar or shortcut
- **THEN** a modal panel shows adb path, buffer configuration, theme, and language controls editable in the terminal with keyboard navigation help visible at the top

#### Scenario: Open tag filter modal from filter row

- **WHEN** the user activates Tag filter edit from the filter row
- **THEN** an in-TUI modal panel provides Tag filter text entry

### Requirement: Language field in Settings modal

The Settings modal SHALL include a Language row that cycles through Auto, English, Simplified Chinese, and Traditional Chinese using the same horizontal-key adjust pattern as Theme. Option labels for the Language row SHALL be fixed as `Auto`, `English`, `简体中文`, and `繁體中文` regardless of the currently active UI locale.

#### Scenario: Adjust language with horizontal keys

- **WHEN** the Settings modal is open, language is focused, and the user presses Left or Right
- **THEN** the language preference cycles backward or forward through Auto, English, Simplified Chinese, and Traditional Chinese

#### Scenario: Language option labels stay fixed

- **WHEN** the Settings modal Language row is rendered while the active UI locale is English
- **THEN** the selectable values are still shown as Auto, English, 简体中文, and 繁體中文

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

The system SHALL allow the user to drag with the left mouse button in the log viewport to select contiguous text across one or more visible formatted log lines. Selection in soft-wrap mode SHALL follow logical log entry character indices, spanning wrapped display rows within an entry. Completing a drag (mouse button up) SHALL leave the selection highlighted and SHALL NOT copy to the clipboard. Copy SHALL use Cmd+C on macOS or Ctrl+C on Windows when a non-empty selection exists.

#### Scenario: Drag select in log viewport

- **WHEN** the user presses left mouse button in the log viewport and drags
- **THEN** the selected character range is highlighted in the log viewport

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

### Requirement: Log viewport editor-like keyboard bindings

While focus is on the log viewport on the top layer, the system SHALL use arrow keys, Home, End, PageUp, and PageDown for caret movement (and Shift variants for selection) per log-display requirements. The system SHALL NOT bind `j` or `k` as log-viewport scroll shortcuts. Existing non-navigation log shortcuts (for example pause, clear, follow, find, filters) SHALL remain available when they do not conflict with these bindings.

#### Scenario: Arrows move caret not viewport-only scroll

- **WHEN** focus is on the log viewport on the top layer and the user presses Up or Down without Shift
- **THEN** the caret moves and the viewport follows only as needed to keep the caret visible, rather than scrolling without a caret

#### Scenario: j and k no longer scroll logs

- **WHEN** focus is on the log viewport on the top layer and the user presses `j` or `k` without modifiers that invoke other shortcuts
- **THEN** the log viewport does not scroll solely because of that key

#### Scenario: Page keys remain available

- **WHEN** focus is on the log viewport on the top layer and the user presses PageUp or PageDown
- **THEN** the caret moves by a page per log-display requirements

### Requirement: Log viewport multi-click selection

The system SHALL interpret consecutive left-button presses in the log viewport within a short time window and near the same position as multi-clicks: one click places the caret and clears selection (and may begin a drag); two clicks select a word; three clicks select the logical line, per log-display word and line selection extents. Multi-click selection SHALL NOT write the clipboard.

#### Scenario: Single click places caret

- **WHEN** the user single-clicks inside the log viewport
- **THEN** the caret moves to the clicked character position and any previous selection is cleared

#### Scenario: Double-click selects word

- **WHEN** the user double-clicks a word character in the log viewport
- **THEN** a word selection is created per log-display rules and the clipboard is not updated

#### Scenario: Triple-click selects line

- **WHEN** the user triple-clicks in the log viewport
- **THEN** the full logical formatted log line is selected and the clipboard is not updated
