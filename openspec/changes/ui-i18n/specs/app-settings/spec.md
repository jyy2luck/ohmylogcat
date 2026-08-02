## ADDED Requirements

### Requirement: Persist language preference

The system SHALL persist the selected language preference (Auto, English, Simplified Chinese, or Traditional Chinese) in settings storage and restore it on the next launch. Missing language fields in existing settings files SHALL default to Auto.

#### Scenario: Language preference persists after restart

- **WHEN** the user sets language to Simplified Chinese, saves settings, and restarts the application
- **THEN** the language preference remains Simplified Chinese

#### Scenario: Missing language field defaults to Auto

- **WHEN** settings are loaded from a file that has no language field
- **THEN** the language preference is Auto

## MODIFIED Requirements

### Requirement: Settings accessible from UI

The system SHALL provide a settings entry point from the main TUI (toolbar control and/or keyboard shortcut) that opens an in-terminal settings panel to edit adb path, buffer configuration, theme, and language preference.

#### Scenario: Open settings

- **WHEN** the user opens settings from the main TUI
- **THEN** adb path, buffer preset, theme, and language controls are displayed in a modal panel and are editable
