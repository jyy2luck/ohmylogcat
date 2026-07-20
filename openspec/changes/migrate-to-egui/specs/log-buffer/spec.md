## MODIFIED Requirements

### Requirement: Maintain a fixed-capacity ring buffer

The system SHALL store received log entries in a fixed-capacity ring buffer; when capacity is exceeded, the oldest entries SHALL be discarded. The buffer SHALL grow on demand up to the configured capacity and MUST NOT pre-allocate storage for the full capacity while empty.

#### Scenario: Buffer reaches capacity

- **WHEN** the number of stored entries exceeds the configured buffer size
- **THEN** the oldest entries are removed and the buffer size remains at the configured maximum

#### Scenario: Empty buffer stays compact

- **WHEN** the application launches or the user clears the buffer
- **THEN** the ring buffer does not retain full-capacity empty slot allocation solely to reserve the configured maximum
