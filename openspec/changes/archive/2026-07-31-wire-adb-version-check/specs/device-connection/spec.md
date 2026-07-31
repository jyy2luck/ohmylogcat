## MODIFIED Requirements

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
