# device-connection Specification

## Purpose
TBD - created by archiving change mvp-logcat-viewer. Update Purpose after archive.
## Requirements
### Requirement: List connected adb devices

The system SHALL enumerate connected Android devices and emulators via `adb devices` and display each entry with serial and state.

#### Scenario: Device connected

- **WHEN** one or more devices are connected and authorized
- **THEN** the device dropdown lists each device serial with status "device"

#### Scenario: No devices connected

- **WHEN** no adb devices are available
- **THEN** the device dropdown shows an empty or placeholder state and log streaming is disabled

### Requirement: Select active device

The system SHALL allow the user to select one device as the active logcat source.

#### Scenario: Switch device

- **WHEN** the user selects a different device from the dropdown
- **THEN** the system stops the current logcat stream and starts streaming from the newly selected device

### Requirement: Detect adb availability

The system SHALL resolve the adb executable path, then verify it by running a version check (`adb --version`) before device discovery and before log streaming. Path resolution alone (file exists) is not sufficient.

#### Scenario: adb not found

- **WHEN** adb is missing or not executable at the configured path
- **THEN** the system displays an error with guidance to install platform-tools or configure the adb path

#### Scenario: adb available

- **WHEN** adb responds successfully to a version check
- **THEN** the system enables device discovery and log streaming

#### Scenario: adb path resolves but version check fails

- **WHEN** a path to adb is found but `adb --version` fails to execute or exits unsuccessfully
- **THEN** the system displays an error and does not proceed with device discovery or log streaming

