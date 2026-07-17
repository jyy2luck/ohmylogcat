## ADDED Requirements

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

### Requirement: Settings accessible from UI

The system SHALL provide a settings entry point from the main window to edit adb path and buffer configuration.

#### Scenario: Open settings

- **WHEN** the user opens settings from the main window
- **THEN** adb path and buffer preset controls are displayed and editable
