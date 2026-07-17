## ADDED Requirements

### Requirement: Maintain a fixed-capacity ring buffer

The system SHALL store received log entries in a fixed-capacity ring buffer in the Rust backend; when capacity is exceeded, the oldest entries SHALL be discarded.

#### Scenario: Buffer reaches capacity

- **WHEN** the number of stored entries exceeds the configured buffer size
- **THEN** the oldest entries are removed and the buffer size remains at the configured maximum

### Requirement: Default buffer size 200,000 lines

The system SHALL default the ring buffer capacity to 200,000 lines (Normal preset).

#### Scenario: First launch defaults

- **WHEN** the application launches for the first time with no saved settings
- **THEN** the buffer capacity is 200,000 lines

### Requirement: Buffer size presets

The system SHALL offer buffer presets: Light (50,000), Normal (200,000), Heavy (500,000), and Marathon (1,000,000), plus a custom line count.

#### Scenario: Select Heavy preset

- **WHEN** the user selects the Heavy preset in settings
- **THEN** the buffer capacity is set to 500,000 lines and persists across restarts

### Requirement: Display buffer usage in status bar

The system SHALL show current entry count, configured maximum, approximate lines per second, and estimated memory usage in a status bar.

#### Scenario: Status bar during streaming

- **WHEN** logs are streaming actively
- **THEN** the status bar displays current count, maximum count, and a live indicator
