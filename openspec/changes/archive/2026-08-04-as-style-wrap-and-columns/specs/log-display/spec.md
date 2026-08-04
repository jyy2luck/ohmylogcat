## ADDED Requirements

### Requirement: Fixed-width PID and TID columns

The system SHALL format each displayed log entry with PID and TID right-aligned in a fixed field width of 5 characters (space-padded), matching common `logcat -v threadtime` / Android Studio column alignment and the existing export format. Level, tag, and message SHALL begin at stable columns across entries that share the same timestamp width.

#### Scenario: Short PID and TID are padded

- **WHEN** a log entry has PID `42` and TID `7`
- **THEN** the formatted display line includes those fields as width-5 padded values (e.g. `   42` and `    7`) so the level character aligns with other rows

#### Scenario: Find and copy use padded formatting

- **WHEN** the user searches or copies text from the log viewport
- **THEN** matches and clipboard contents use the same padded formatted line text as shown in the viewport (hanging-indent display spaces on wrap continuations are not part of that logical text)

### Requirement: Soft-Wrap hanging indent under message column

When Soft-Wrap is on, the system SHALL wrap long formatted log lines so that the first display row of an entry starts at column 0, and each subsequent soft-wrapped display row of that entry is indented to the start of the **message** field (the character immediately after `{tag}: ` in the formatted line). Hanging-indent leading spaces on continuation rows are display-only: they MUST NOT be included in logical character indices, find matches, selection ranges, or copied text. When Soft-Wrap is off, this hanging indent does not apply.

#### Scenario: Continuation aligns under message

- **WHEN** Soft-Wrap is on and a formatted log line exceeds the viewport width
- **THEN** the second and later display rows for that entry begin visually under the message column, not under the timestamp

#### Scenario: Hang indent ignored when Soft-Wrap is off

- **WHEN** Soft-Wrap is off and a log line exceeds the viewport width
- **THEN** the entry remains a single display row with horizontal pan and no hanging indent

#### Scenario: Narrow viewport falls back safely

- **WHEN** Soft-Wrap is on and the computed message-column indent is greater than or equal to the viewport width
- **THEN** the system wraps without hanging indent (continuations start at column 0) rather than producing empty or unusable content rows

#### Scenario: Interaction stays on logical text

- **WHEN** Soft-Wrap is on with hanging indent and the user moves the caret, selects text, or clicks in the log viewport
- **THEN** positions map to character indices within the formatted logical line (excluding hang-indent pad spaces), and clicking within a continuation’s pad area maps to the first logical character of that wrap chunk

## MODIFIED Requirements

### Requirement: Soft-Wrap toggle for log lines

The system SHALL support a Soft-Wrap preference. When Soft-Wrap is off, each log entry SHALL render on a single terminal row with horizontal panning or truncation such that the full line remains reachable. When Soft-Wrap is on, long lines SHALL wrap within the viewport width using message-column hanging indent as specified by Soft-Wrap hanging indent under message column; exact variable-height virtualization quality remains best-effort in the TUI shell.

#### Scenario: Default is no wrap

- **WHEN** the user opens the application for the first time or has no saved preference
- **THEN** Soft-Wrap is off and each log entry occupies a single row in the viewport

#### Scenario: No wrap with horizontal access

- **WHEN** Soft-Wrap is off and a log line exceeds the viewport width
- **THEN** the user can pan horizontally or otherwise reveal the clipped portion of the line

#### Scenario: Persist soft wrap preference

- **WHEN** the user toggles Soft-Wrap (when the control is available)
- **THEN** the preference is persisted across application restarts

#### Scenario: Wrap uses hanging indent

- **WHEN** Soft-Wrap is on and a log line wraps across multiple display rows
- **THEN** continuation rows are hanging-indented under the message column per Soft-Wrap hanging indent under message column
