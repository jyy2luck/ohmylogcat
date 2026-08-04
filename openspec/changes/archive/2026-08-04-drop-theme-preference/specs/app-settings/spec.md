## MODIFIED Requirements

### Requirement: Settings accessible from UI

The system SHALL provide a settings entry point from the main TUI (toolbar control and/or keyboard shortcut) that opens an in-terminal settings panel to edit adb path, buffer configuration, and language preference.

#### Scenario: Open settings

- **WHEN** the user opens settings from the main TUI
- **THEN** adb path, buffer preset, and language controls are displayed in a modal panel and are editable

### Requirement: Settings persist immediately on change in modal

When the Settings modal is open, the system SHALL persist each settings field to storage as soon as the user changes it: cycle-type fields (buffer preset, language) on each horizontal adjust, and text fields (adb path, custom capacity) after each edit that changes the stored value. Closing the Settings modal with Enter or Esc SHALL NOT be required to save changes.

#### Scenario: Buffer preset persists on horizontal adjust

- **WHEN** the Settings modal is open, buffer preset is focused, and the user cycles to Heavy with Left or Right
- **THEN** the buffer capacity corresponding to Heavy is written to settings storage before the modal closes

#### Scenario: Adb path persists on text edit

- **WHEN** the Settings modal is open, ADB path is focused, and the user types characters that change the path value
- **THEN** the adb path is written to settings storage after the edit without requiring Enter

#### Scenario: Dismiss does not require save action

- **WHEN** the user adjusts one or more settings fields and presses Esc or Enter to close the Settings modal
- **THEN** all changes made during the session remain persisted and are restored on next launch

## ADDED Requirements

### Requirement: Ignore legacy theme field

When loading settings, the system SHALL ignore a legacy `theme` field if present and SHALL NOT write a `theme` field on subsequent saves.

#### Scenario: Old settings file with theme loads

- **WHEN** settings are loaded from a file that still contains a `theme` value
- **THEN** the application starts successfully without exposing a theme preference, and the next save omits `theme`
