## ADDED Requirements

### Requirement: Log viewport editor-like keyboard bindings

While focus is on the log viewport on the top layer, the system SHALL use arrow keys, Home, End, PageUp, and PageDown for caret movement (and Shift variants for selection) per log-display requirements. The system SHALL NOT bind `j` or `k` as log-viewport scroll shortcuts. Existing non-navigation log shortcuts (for example pause, clear, follow, find, filters) SHALL remain available when they do not conflict with these bindings.

#### Scenario: Arrows move caret not viewport-only scroll

- **WHEN** focus is on the log viewport on the top layer and the user presses Up or Down without Shift
- **THEN** the caret moves and the viewport follows only as needed to keep the caret visible, rather than scrolling without a caret

#### Scenario: j and k no longer scroll logs

- **WHEN** focus is on the log viewport on the top layer and the user presses `j` or `k` without modifiers that invoke other shortcuts
- **THEN** the log viewport does not scroll solely because of that key

#### Scenario: Page keys remain available

- **WHEN** focus is on the log viewport on the top layer and the user presses PageUp or PageDown
- **THEN** the caret moves by a page per log-display requirements

### Requirement: Log viewport multi-click selection

The system SHALL interpret consecutive left-button presses in the log viewport within a short time window and near the same position as multi-clicks: one click places the caret and clears selection (and may begin a drag); two clicks select a word; three clicks select the logical line, per log-display word and line selection extents. Multi-click selection SHALL NOT write the clipboard.

#### Scenario: Single click places caret

- **WHEN** the user single-clicks inside the log viewport
- **THEN** the caret moves to the clicked character position and any previous selection is cleared

#### Scenario: Double-click selects word

- **WHEN** the user double-clicks a word character in the log viewport
- **THEN** a word selection is created per log-display rules and the clipboard is not updated

#### Scenario: Triple-click selects line

- **WHEN** the user triple-clicks in the log viewport
- **THEN** the full logical formatted log line is selected and the clipboard is not updated

## MODIFIED Requirements

### Requirement: Log viewport mouse text selection

The system SHALL allow the user to drag with the left mouse button in the log viewport to select contiguous text across one or more visible formatted log lines. Selection in soft-wrap mode SHALL follow logical log entry character indices, spanning wrapped display rows within an entry. Completing a drag (mouse button up) SHALL leave the selection highlighted and SHALL NOT copy to the clipboard. Copy SHALL use Cmd+C on macOS or Ctrl+C on Windows when a non-empty selection exists.

#### Scenario: Drag select in log viewport

- **WHEN** the user presses left mouse button in the log viewport and drags
- **THEN** the selected character range is highlighted in the log viewport

#### Scenario: Clear selection outside logs

- **WHEN** the user clicks outside the log viewport
- **THEN** any active selection is cleared

#### Scenario: Mouse-up does not copy

- **WHEN** the user finishes a non-empty drag selection by releasing the left mouse button
- **THEN** the selection remains visible and the system clipboard is not modified by the mouse-up

#### Scenario: Copy selection shortcut

- **WHEN** a non-empty selection exists and the user presses Cmd+C on macOS or Ctrl+C on Windows
- **THEN** the selected plain text is copied to the system clipboard

#### Scenario: Copy does not quit

- **WHEN** the user presses Ctrl+C with no active selection
- **THEN** the application does not quit (existing Ctrl+C non-quit behavior preserved)
