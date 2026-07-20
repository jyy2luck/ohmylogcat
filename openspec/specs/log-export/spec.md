# log-export Specification

## Purpose
TBD - created by archiving change mvp-logcat-viewer. Update Purpose after archive.

## Requirements

### Requirement: Export filtered logs

The system SHALL allow exporting the currently filtered log entries to a text file in threadtime format. The destination path SHALL be chosen via an in-TUI path prompt (with a sensible default filename), not a native OS GUI file dialog.

#### Scenario: Export filtered results

- **WHEN** the user activates Export with active filters and confirms a destination path in the TUI prompt
- **THEN** the system saves only entries matching the current filters to that file

### Requirement: Export all buffered logs

The system SHALL allow exporting all entries in the ring buffer regardless of active filters when the user selects export-all. The destination path SHALL be chosen via an in-TUI path prompt (with a sensible default filename).

#### Scenario: Export all buffer

- **WHEN** the user chooses to export all buffered logs and confirms a destination path in the TUI prompt
- **THEN** the system saves every entry in the ring buffer to that file in threadtime format
