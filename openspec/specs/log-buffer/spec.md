# log-buffer Specification

## Purpose
TBD - created by archiving change mvp-logcat-viewer. Update Purpose after archive.
## Requirements
### Requirement: Maintain a fixed-capacity ring buffer

The system SHALL store received log entries in a fixed-capacity ring buffer; when capacity is exceeded, the oldest entries SHALL be discarded. The buffer SHALL grow on demand up to the configured capacity and MUST NOT pre-allocate storage for the full capacity while empty.

#### Scenario: Buffer reaches capacity

- **WHEN** the number of stored entries exceeds the configured buffer size
- **THEN** the oldest entries are removed and the buffer size remains at the configured maximum

#### Scenario: Empty buffer stays compact

- **WHEN** the application launches or the user clears the buffer
- **THEN** the ring buffer does not retain full-capacity empty slot allocation solely to reserve the configured maximum

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

The system SHALL show in the status bar: a session-state indicator shared with the toolbar (streaming / paused / disconnected), filtered entry count, and stored entry count. The status bar SHALL NOT show focus-target, wrap-state, throughput, memory, or parenthetical legends. Errors and ephemeral copy/export messages SHALL occupy the right side of the status row rather than appending to the count cluster. The streaming indicator SHALL use wording that means streaming/pulling logs rather than a generic "live/real-time" label, and SHALL match the toolbar session label.

#### Scenario: Status bar during streaming

- **WHEN** logs are streaming actively, display is not paused, and the UI language is English
- **THEN** both the toolbar and the status bar show a Streaming session label and the status bar shows `filtered / stored` without a max/rate/memory legend

#### Scenario: Status bar labels follow UI language
- **WHEN** the UI language is Simplified Chinese
- **THEN** the session indicator uses 拉流中, 已暂停, or 已断开 (matching the toolbar), and the count pair is `筛选数 / 已存` without parenthetical legends

#### Scenario: Status bar Traditional Chinese labels
- **WHEN** the UI language is Traditional Chinese
- **THEN** the session indicator uses 拉流中, 已暫停, or 已斷開 (matching the toolbar)

#### Scenario: Paused agrees across chrome
- **WHEN** streaming is paused
- **THEN** the toolbar session label and the status bar session label both read as paused (not Streaming / 拉流中)

