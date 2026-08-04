## MODIFIED Requirements

### Requirement: Settings modal keyboard navigation

The Settings modal SHALL use vertical arrow keys to move focus among visible settings rows and horizontal arrow keys to adjust cycle-type fields (buffer preset and language). Adjustments and text edits SHALL apply immediately to runtime state and persist to settings storage without requiring Enter. The modal SHALL display a help line at the top documenting move, adjust, text entry, and dismiss bindings. Focus SHALL always rest on a visible row; the Custom capacity row SHALL be included in navigation only when the buffer preset is Custom. Pressing Enter or Esc SHALL close the modal and return focus to the log viewport without an additional save or cancel step.

#### Scenario: Move focus with vertical keys

- **WHEN** the Settings modal is open and the user presses Up or Down
- **THEN** the focus cursor moves to the previous or next visible settings row and the `>` marker follows the focused row

#### Scenario: Adjust preset with horizontal keys

- **WHEN** the Settings modal is open, buffer preset is focused, and the user presses Left or Right
- **THEN** the buffer preset cycles backward or forward, buffer capacity updates immediately, and the new value is persisted

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
- **THEN** focus moves to an adjacent visible row (preset or language) and the Custom row is no longer focused

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
- **THEN** a modal panel shows adb path, buffer configuration, and language controls editable in the terminal with keyboard navigation help visible at the top

#### Scenario: Open tag filter modal from filter row

- **WHEN** the user activates Tag filter edit from the filter row
- **THEN** an in-TUI modal panel provides Tag filter text entry

### Requirement: Language field in Settings modal

The Settings modal SHALL include a Language row that cycles through Auto, English, Simplified Chinese, and Traditional Chinese using the same horizontal-key adjust pattern as other cycle-type settings rows. Option labels for the Language row SHALL be fixed as `Auto`, `English`, `简体中文`, and `繁體中文` regardless of the currently active UI locale.

#### Scenario: Adjust language with horizontal keys

- **WHEN** the Settings modal is open, language is focused, and the user presses Left or Right
- **THEN** the language preference cycles backward or forward through Auto, English, Simplified Chinese, and Traditional Chinese

#### Scenario: Language option labels stay fixed

- **WHEN** the Settings modal Language row is rendered while the active UI locale is English
- **THEN** the selectable values are still shown as Auto, English, 简体中文, and 繁體中文
