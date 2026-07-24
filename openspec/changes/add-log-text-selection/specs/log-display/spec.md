## ADDED Requirements

### Requirement: Selection highlight distinct from find

The system SHALL render log viewport text selection with a visual style distinct from find-match highlighting so both can be recognized when find is open.

#### Scenario: Selection visible with find active

- **WHEN** find highlights are shown and the user selects log text
- **THEN** the selection highlight remains visually distinct from find match highlights

### Requirement: Multi-line selection copy text

The system SHALL copy selected text as plain characters from formatted log lines, inserting newline characters between selected log entries.

#### Scenario: Copy spans multiple log entries

- **WHEN** the user selects text across two or more log entries and copies
- **THEN** the clipboard contains the formatted line text joined by newline characters in log order
