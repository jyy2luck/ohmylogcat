# log-streaming Specification

## Purpose
TBD - created by archiving change mvp-logcat-viewer. Update Purpose after archive.
## Requirements
### Requirement: Stream logcat in threadtime format

The system SHALL stream logs from the active device using `adb logcat -v threadtime` and parse each line into structured fields: timestamp, pid, tid, level, tag, and message.

#### Scenario: Live stream starts

- **WHEN** a device is selected and streaming is active
- **THEN** new log lines appear in the log view within one second of device emission under normal load

#### Scenario: Parse valid log line

- **WHEN** a well-formed threadtime line is received
- **THEN** the system populates timestamp, pid, tid, level, tag, and message on the log entry

### Requirement: Pause and resume streaming

The system SHALL provide Pause and Resume controls that stop and restart display of new logs without terminating the adb process when possible.

#### Scenario: Pause streaming

- **WHEN** the user clicks Pause
- **THEN** new log lines are buffered but not appended to the visible list until Resume

#### Scenario: Resume streaming

- **WHEN** the user clicks Resume after Pause
- **THEN** buffered and subsequent log lines are displayed again

### Requirement: Clear log view

The system SHALL provide a Clear control that empties the in-memory log buffer and the visible log list.

#### Scenario: Clear logs

- **WHEN** the user clicks Clear
- **THEN** the ring buffer and visible list are emptied and the buffer usage indicator resets to zero

