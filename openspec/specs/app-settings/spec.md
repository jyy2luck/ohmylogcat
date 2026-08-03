# app-settings Specification

## Purpose
TBD - created by archiving change mvp-logcat-viewer. Update Purpose after archive.

## Requirements

### Requirement: Configure adb executable path

The system SHALL allow the user to set the absolute path to the adb executable and persist it across sessions.

#### Scenario: Custom adb path on Windows

- **WHEN** the user sets adb path to `%LOCALAPPDATA%\Android\Sdk\platform-tools\adb.exe`
- **THEN** subsequent device discovery and log streaming use that executable

#### Scenario: Default adb path

- **WHEN** no custom adb path is configured
- **THEN** the system attempts to use `adb` from PATH, with platform-specific common SDK locations as fallback hints only

### Requirement: Persist buffer preset

The system SHALL persist the selected buffer preset or custom line count and restore it on next launch.

#### Scenario: Settings persist after restart

- **WHEN** the user changes buffer preset to Heavy and restarts the application
- **THEN** the buffer capacity remains 500,000 lines

### Requirement: Persist language preference

The system SHALL persist the selected language preference (Auto, English, Simplified Chinese, or Traditional Chinese) in settings storage and restore it on the next launch. Missing language fields in existing settings files SHALL default to Auto.

#### Scenario: Language preference persists after restart

- **WHEN** the user sets language to Simplified Chinese, saves settings, and restarts the application
- **THEN** the language preference remains Simplified Chinese

#### Scenario: Missing language field defaults to Auto

- **WHEN** settings are loaded from a file that has no language field
- **THEN** the language preference is Auto

### Requirement: Settings accessible from UI

The system SHALL provide a settings entry point from the main TUI (toolbar control and/or keyboard shortcut) that opens an in-terminal settings panel to edit adb path, buffer configuration, theme, and language preference.

#### Scenario: Open settings

- **WHEN** the user opens settings from the main TUI
- **THEN** adb path, buffer preset, theme, and language controls are displayed in a modal panel and are editable

### Requirement: Settings persist immediately on change in modal

When the Settings modal is open, the system SHALL persist each settings field to storage as soon as the user changes it: cycle-type fields (buffer preset, theme, language) on each horizontal adjust, and text fields (adb path, custom capacity) after each edit that changes the stored value. Closing the Settings modal with Enter or Esc SHALL NOT be required to save changes.

#### Scenario: Theme persists on horizontal adjust

- **WHEN** the Settings modal is open, theme is focused, and the user cycles to a new theme with Left or Right
- **THEN** the theme preference is written to settings storage before the modal closes

#### Scenario: Buffer preset persists on horizontal adjust

- **WHEN** the Settings modal is open, buffer preset is focused, and the user cycles to Heavy with Left or Right
- **THEN** the buffer capacity corresponding to Heavy is written to settings storage before the modal closes

#### Scenario: Adb path persists on text edit

- **WHEN** the Settings modal is open, ADB path is focused, and the user types characters that change the path value
- **THEN** the adb path is written to settings storage after the edit without requiring Enter

#### Scenario: Dismiss does not require save action

- **WHEN** the user adjusts one or more settings fields and presses Esc or Enter to close the Settings modal
- **THEN** all changes made during the session remain persisted and are restored on next launch
