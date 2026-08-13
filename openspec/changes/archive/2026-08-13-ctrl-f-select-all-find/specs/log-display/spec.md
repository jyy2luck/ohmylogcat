## MODIFIED Requirements

### Requirement: Find in log with keyboard shortcut

The system SHALL provide an in-view find UI opened by `/` and, when the terminal delivers them, Cmd+F on macOS or Ctrl+F on Windows, that searches within currently visible (filter-applied) log entries without hiding non-matching lines. When no modal is open, Ctrl+F / Cmd+F SHALL always give keyboard focus to the Find input, including when Find is already focused. When that shortcut runs and the current query is non-empty, the system SHALL select the entire query. `/` SHALL continue to open or re-focus Find from the log viewport with the insertion cursor at the end of the current query and SHALL NOT select the query.

#### Scenario: Open find bar with slash

- **WHEN** the user presses `/` from the log viewport
- **THEN** a find input appears (if it was closed) and receives keyboard focus with the insertion cursor at the end of the current query

#### Scenario: Slash does not select the query

- **WHEN** Find is open with a non-empty query, focus is on the log viewport, and the user presses `/`
- **THEN** Find receives keyboard focus and the query is not selected (insertion cursor at the end)

#### Scenario: Ctrl+F opens find

- **WHEN** no modal is open, Find is closed, and the user presses Ctrl+F or Cmd+F (when delivered by the terminal)
- **THEN** a find input appears and receives keyboard focus

#### Scenario: Ctrl+F focuses find from the log viewport

- **WHEN** no modal is open, Find is open with a non-empty query, focus is on the log viewport, and the user presses Ctrl+F or Cmd+F
- **THEN** focus moves to the Find input and the entire current query is selected

#### Scenario: Ctrl+F selects query while Find is focused

- **WHEN** Find is open and focused with a non-empty query and the user presses Ctrl+F or Cmd+F
- **THEN** the Find input remains focused and the entire current query is selected

#### Scenario: Ctrl+F from Level control

- **WHEN** no modal is open, focus is on the Level control, and the user presses Ctrl+F or Cmd+F
- **THEN** the Find bar opens if needed, receives focus, and the current query is selected if non-empty

#### Scenario: Ctrl+F ignored while a modal is open

- **WHEN** a modal is open and the user presses Ctrl+F or Cmd+F
- **THEN** the Find bar does not steal focus and the key is handled by the modal as today

#### Scenario: Case insensitive search

- **WHEN** the user enters a search query in the find input
- **THEN** the system matches substrings case-insensitively within the formatted text of each visible log entry

#### Scenario: Highlight all matches

- **WHEN** one or more matches exist for the current query
- **THEN** matching substrings are visually highlighted in the log viewport (e.g. ANSI emphasis)

#### Scenario: No matches

- **WHEN** the query matches no visible log entries
- **THEN** the find UI shows zero matches and no highlights are displayed

#### Scenario: Find does not filter logs

- **WHEN** the user searches with the find UI
- **THEN** all log entries remain visible and only matching substrings are highlighted
