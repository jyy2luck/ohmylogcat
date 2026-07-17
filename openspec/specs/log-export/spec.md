# log-export Specification

## Purpose
TBD - created by archiving change mvp-logcat-viewer. Update Purpose after archive.
## Requirements
### Requirement: Export filtered logs

The system SHALL allow exporting the currently filtered log entries to a text file in threadtime format.

#### Scenario: Export filtered results

- **WHEN** the user clicks Export with active filters
- **THEN** the system saves only entries matching the current filters to a user-chosen file

### Requirement: Export all buffered logs

The system SHALL allow exporting all entries in the ring buffer regardless of active filters when the user selects export-all.

#### Scenario: Export all buffer

- **WHEN** the user chooses to export all buffered logs
- **THEN** the system saves every entry in the ring buffer to a user-chosen file in threadtime format

