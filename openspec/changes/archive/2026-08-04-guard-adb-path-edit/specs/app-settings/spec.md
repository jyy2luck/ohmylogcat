## MODIFIED Requirements

### Requirement: Settings accessible from UI

The system SHALL provide a settings entry point from the main TUI (toolbar control and/or keyboard shortcut) that opens an in-terminal settings panel to view and configure adb path, buffer configuration, and language preference. The adb path row SHALL be viewable without unlocking and editable only after the user explicitly enters ADB path edit mode.

#### Scenario: Open settings

- **WHEN** the user opens settings from the main TUI
- **THEN** adb path, buffer preset, and language controls are displayed in a modal panel; adb path starts locked (not directly typeable), and keyboard navigation help is visible at the top

### Requirement: Settings persist immediately on change in modal

When the Settings modal is open, the system SHALL persist each settings field to storage as soon as the user changes it: cycle-type fields (buffer preset, language) on each horizontal adjust, Custom capacity after each text edit that changes the stored value, and adb path after each text edit while ADB path edit mode is active or when restore-to-Auto clears the custom path. Closing the Settings modal with Enter or Esc SHALL NOT be required to save changes.

#### Scenario: Buffer preset persists on horizontal adjust

- **WHEN** the Settings modal is open, buffer preset is focused, and the user cycles to Heavy with Left or Right
- **THEN** the buffer capacity corresponding to Heavy is written to settings storage before the modal closes

#### Scenario: Adb path persists on text edit in edit mode

- **WHEN** the Settings modal is open, ADB path is focused and unlocked for editing, and the user types characters that change the path value
- **THEN** the adb path is written to settings storage after the edit without requiring Enter

#### Scenario: Restore Auto persists immediately

- **WHEN** the Settings modal is open, ADB path is focused and locked, a custom path is configured, and the user restores Auto
- **THEN** the stored adb path is cleared (Auto) without requiring Enter

#### Scenario: Dismiss does not require save action

- **WHEN** the user adjusts one or more settings fields and presses Esc or Enter to close the Settings modal
- **THEN** all changes made during the session remain persisted and are restored on next launch

## ADDED Requirements

### Requirement: Guarded ADB path editing

The system SHALL keep the ADB path setting locked against direct typing by default. While locked and focused, printable character input and Backspace SHALL NOT modify the path. The user SHALL unlock editing with an explicit edit action before typing is accepted. While unlocked, append-only typing and Backspace SHALL modify the path and persist as for other text settings. Exiting edit mode SHALL keep the current path value and SHALL NOT dismiss the Settings modal.

#### Scenario: Locked path ignores typing

- **WHEN** the Settings modal is open, ADB path is focused and locked, and the user types a printable character or presses Backspace
- **THEN** the adb path value and settings storage remain unchanged

#### Scenario: Unlock then edit

- **WHEN** the Settings modal is open, ADB path is focused and locked, and the user activates edit
- **THEN** ADB path enters edit mode and subsequent printable characters and Backspace modify the path

#### Scenario: Exit edit keeps value

- **WHEN** ADB path edit mode is active and the user exits edit mode
- **THEN** edit mode ends, the current path value is retained (including any edits already persisted), and the Settings modal remains open

### Requirement: Restore ADB path to Auto

The system SHALL allow the user to clear a custom adb path and return to automatic resolution from the Settings modal without manually deleting the path character by character. Restoring Auto SHALL persist immediately.

#### Scenario: Restore Auto from locked custom path

- **WHEN** the Settings modal is open, ADB path is focused and locked, a custom path is stored, and the user restores Auto
- **THEN** the custom path is cleared, subsequent device discovery and streaming use automatic adb resolution, and the change is persisted

#### Scenario: Restore Auto while already Auto

- **WHEN** the Settings modal is open, ADB path is focused and locked, no custom path is stored, and the user restores Auto
- **THEN** the path remains Auto and settings storage stays unchanged
