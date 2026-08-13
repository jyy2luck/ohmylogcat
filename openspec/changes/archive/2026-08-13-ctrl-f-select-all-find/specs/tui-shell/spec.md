## ADDED Requirements

### Requirement: Find query select-all

The system SHALL select the entire Find query when the user activates Ctrl+F or Cmd+F with no modal open and the query is non-empty. Selected query text SHALL be visually distinct from unselected Find-bar chrome (prefix, brackets, match counter, and help). While the query is selected: typing a printable character SHALL replace the entire query with that character and collapse the selection; Backspace or Delete SHALL clear the query and collapse the selection; Enter and Shift+Enter SHALL navigate to the next or previous match without collapsing the selection; Left, Right, Home, End, or a mouse click in the Find input value region SHALL collapse the selection to a normal insertion cursor. An empty query SHALL receive focus with no selection. Tag filter, Message filter, export path, and Settings text fields SHALL NOT gain select-all from this shortcut.

#### Scenario: Typing replaces selected query

- **WHEN** the Find query `foo` is selected and the user types `b`
- **THEN** the query becomes `b`, the selection is collapsed, and matches are recomputed

#### Scenario: Backspace clears selected query

- **WHEN** the Find query is selected and non-empty and the user presses Backspace
- **THEN** the query becomes empty, the selection is collapsed, and match highlights are cleared

#### Scenario: Enter keeps selection and goes to next match

- **WHEN** the Find query is selected with matches and the user presses Enter
- **THEN** the view goes to the next match and the query remains selected

#### Scenario: Arrow collapses selection

- **WHEN** the Find query is selected and the user presses Left or Right
- **THEN** the selection is collapsed to a normal insertion cursor and the query text is unchanged

#### Scenario: Click collapses selection

- **WHEN** the Find query is selected and the user clicks inside the Find input value region
- **THEN** the selection is collapsed and the insertion cursor moves to the character index corresponding to the click column

#### Scenario: Empty query has no selection

- **WHEN** the Find query is empty and the user presses Ctrl+F or Cmd+F with no modal open
- **THEN** the Find input is focused and no query selection is shown

#### Scenario: Slash does not select-all

- **WHEN** Find is focused with a non-empty query and the user types `/`
- **THEN** `/` is inserted at the cursor as a query character and does not select the query
