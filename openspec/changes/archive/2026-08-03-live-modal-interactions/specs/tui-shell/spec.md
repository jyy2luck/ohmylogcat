## MODIFIED Requirements

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
