# log-filtering Specification

## Purpose
TBD - created by archiving change mvp-logcat-viewer. Update Purpose after archive.
## Requirements
### Requirement: Filter by tag substring

The system SHALL filter log entries by tag using case-sensitive substring matching when the Tag filter field is non-empty.

#### Scenario: Tag filter match

- **WHEN** the Tag filter is set to "OkHttp"
- **THEN** only entries whose tag contains "OkHttp" are shown

#### Scenario: Tag filter empty

- **WHEN** the Tag filter field is empty
- **THEN** the tag filter imposes no restriction

### Requirement: Filter by message keyword

The system SHALL filter log entries by message content using case-insensitive substring matching when the Message filter field is non-empty.

#### Scenario: Message keyword match

- **WHEN** the Message filter is set to "timeout"
- **THEN** only entries whose message contains "timeout" or "Timeout" (any case) are shown

#### Scenario: Message filter empty

- **WHEN** the Message filter field is empty
- **THEN** the message filter imposes no restriction

### Requirement: Filter by minimum log level

The system SHALL filter log entries to show only those at or above the selected minimum level: Verbose, Debug, Info, Warn, or Error.

#### Scenario: Minimum level Warn

- **WHEN** the Level filter is set to Warn
- **THEN** only entries with level Warn, Error, or Fatal are shown

#### Scenario: Level All

- **WHEN** the Level filter is set to All
- **THEN** entries of all levels are shown

### Requirement: Combine filters with AND logic

The system SHALL apply Tag, Message, and Level filters together with AND logic.

#### Scenario: Combined tag and message filter

- **WHEN** Tag is "OkHttp" and Message is "timeout"
- **THEN** only entries matching both conditions are shown

### Requirement: Apply filters without restarting stream

The system SHALL re-apply filters immediately when any filter field changes without restarting the adb logcat process.

#### Scenario: Change filter while streaming

- **WHEN** the user edits the Message filter during an active stream
- **THEN** the visible list updates to reflect the new filter within 500 ms for buffers up to 200,000 lines

