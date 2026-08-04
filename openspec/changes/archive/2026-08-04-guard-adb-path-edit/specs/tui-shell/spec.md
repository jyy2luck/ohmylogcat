## MODIFIED Requirements

### Requirement: Settings modal keyboard navigation

The Settings modal SHALL use vertical arrow keys to move focus among visible settings rows and horizontal arrow keys to adjust cycle-type fields (buffer preset and language). Adjustments and accepted text edits SHALL apply immediately to runtime state and persist to settings storage without requiring Enter. The ADB path row SHALL be locked by default: while locked and focused, `e` enters edit mode and `r` restores Auto (clears custom path); printable typing and Backspace SHALL NOT change the path. While ADB path edit mode is active, printable characters and Backspace edit the path append-only; `e` and `r` are treated as ordinary path characters. Custom capacity SHALL remain directly typeable when focused. Left/Right SHALL NOT change text field values. The modal SHALL display a help line at the top documenting move, adjust, ADB lock/edit/restore bindings, Custom capacity text entry, and dismiss bindings. Focus SHALL always rest on a visible row; the Custom capacity row SHALL be included in navigation only when the buffer preset is Custom. Pressing Enter SHALL close the modal. Pressing Esc while ADB path edit mode is active SHALL exit edit mode, keep the current path value, and leave the modal open; pressing Esc while ADB path is not in edit mode SHALL close the modal. Closing SHALL return focus to the log viewport without an additional save or cancel step.

#### Scenario: Move focus with vertical keys

- **WHEN** the Settings modal is open and the user presses Up or Down
- **THEN** the focus cursor moves to the previous or next visible settings row and the `>` marker follows the focused row

#### Scenario: Adjust preset with horizontal keys

- **WHEN** the Settings modal is open, buffer preset is focused, and the user presses Left or Right
- **THEN** the buffer preset cycles backward or forward, buffer capacity updates immediately, and the new value is persisted

#### Scenario: Adjust language with horizontal keys

- **WHEN** the Settings modal is open, language is focused, and the user presses Left or Right
- **THEN** the language preference cycles backward or forward, UI chrome updates immediately, and the new value is persisted

#### Scenario: Locked ADB path ignores typing

- **WHEN** the Settings modal is open, ADB path is focused and locked, and the user types a printable character other than `e`/`r` or presses Backspace
- **THEN** the path value is unchanged and is not rewritten to settings storage

#### Scenario: Enter ADB path edit with e

- **WHEN** the Settings modal is open, ADB path is focused and locked, and the user presses `e`
- **THEN** ADB path enters edit mode and the Settings modal remains open

#### Scenario: Restore Auto with r while locked

- **WHEN** the Settings modal is open, ADB path is focused and locked, and the user presses `r`
- **THEN** any custom path is cleared to Auto, the change is persisted immediately, and edit mode remains inactive

#### Scenario: Edit ADB path while unlocked

- **WHEN** the Settings modal is open, ADB path is focused and in edit mode, and the user types printable characters or presses Backspace
- **THEN** the path is edited append-only, Left/Right do not change the path, and the updated value is persisted after the edit

#### Scenario: e and r are literal in edit mode

- **WHEN** the Settings modal is open, ADB path is focused and in edit mode, and the user presses `e` or `r`
- **THEN** the character is appended to the path and persisted (it does not toggle lock or restore Auto)

#### Scenario: Custom capacity still uses direct typing

- **WHEN** the Settings modal is open, Custom capacity is focused, and the user types digits or presses Backspace
- **THEN** the focused field is edited append-only and the updated value is persisted after the edit

#### Scenario: Esc exits ADB edit mode without closing

- **WHEN** the Settings modal is open, ADB path is in edit mode, and the user presses Esc
- **THEN** edit mode ends, the current path value is kept, and the Settings modal remains open

#### Scenario: Custom row hidden from navigation

- **WHEN** the Settings modal is open and the buffer preset is not Custom
- **THEN** Up/Down navigation skips the Custom capacity row and focus never rests on a row that is not rendered

#### Scenario: Focus re-anchors when Custom row hides

- **WHEN** the Settings modal is open, Custom capacity is focused, and the user cycles preset away from Custom
- **THEN** focus moves to an adjacent visible row (preset or language) and the Custom row is no longer focused

#### Scenario: Help line documents controls

- **WHEN** the Settings modal is rendered
- **THEN** the first content line documents vertical move, horizontal adjust with immediate apply, ADB `e`/`r`/Esc-edit bindings, Custom capacity text entry, and Enter/Esc dismiss

#### Scenario: Enter dismisses settings modal

- **WHEN** the Settings modal is open and the user presses Enter
- **THEN** the modal closes and focus returns to the log viewport without reverting settings already applied and persisted

#### Scenario: Esc dismisses settings modal when not editing ADB

- **WHEN** the Settings modal is open, ADB path is not in edit mode, and the user presses Esc
- **THEN** the modal closes and focus returns to the log viewport without reverting settings already applied and persisted

### Requirement: Modal panels for settings and path prompts

The system SHALL present Settings, export path entry, and Tag/Message filter editing as in-TUI modal panels rather than native OS GUI dialogs. The Settings modal SHALL follow the Settings modal keyboard navigation requirement.

#### Scenario: Open settings modal

- **WHEN** the user opens Settings from the toolbar or shortcut
- **THEN** a modal panel shows adb path (locked by default), buffer configuration, and language controls with keyboard navigation help visible at the top

#### Scenario: Open tag filter modal from filter row

- **WHEN** the user activates Tag filter edit from the filter row
- **THEN** an in-TUI modal panel provides Tag filter text entry
