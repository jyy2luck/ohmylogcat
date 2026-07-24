## ADDED Requirements

### Requirement: Text input insertion cursor

The system SHALL maintain a visible text insertion cursor for Tag filter modal, Message filter modal, and Find bar text inputs. Each surface SHALL track an independent cursor index as a character offset into its string. When a text input surface opens, the cursor SHALL be placed at the end of the current string (index equal to string length, or zero when empty). While a text input is focused, the system SHALL show the terminal hardware cursor at the insertion point using a blinking I-beam style by default. Cursor movement alone SHALL NOT trigger filter or find recomputation; only changes to the input string SHALL.

#### Scenario: Cursor at end when opening tag filter modal

- **WHEN** the Tag filter modal opens and the current Tag filter is `myapp`
- **THEN** the insertion cursor is positioned after the last character of `myapp`

#### Scenario: Move cursor with arrow keys

- **WHEN** a Tag filter, Message filter, or Find text input is focused and the user presses Left or Right
- **THEN** the insertion cursor moves one character within bounds without changing the string

#### Scenario: Home and End keys

- **WHEN** a text input listed in this requirement is focused and the user presses Home or End
- **THEN** the insertion cursor moves to the start or end of the string respectively

#### Scenario: Insert at cursor

- **WHEN** a text input listed in this requirement is focused with the cursor not at the end and the user types a printable character
- **THEN** the character is inserted at the cursor and the cursor advances one position

#### Scenario: Backspace at cursor

- **WHEN** a text input listed in this requirement is focused with the cursor after at least one character and the user presses Backspace
- **THEN** the character immediately before the cursor is removed and the cursor moves back one position

#### Scenario: Delete at cursor

- **WHEN** a text input listed in this requirement is focused with the cursor before the end of the string and the user presses Delete
- **THEN** the character at the cursor is removed and the cursor index is unchanged

#### Scenario: Mouse click positions cursor

- **WHEN** the user clicks inside the Tag filter, Message filter, or Find input value region
- **THEN** the insertion cursor moves to the character index corresponding to the click column, clamped to the string bounds

#### Scenario: Find match navigation unchanged

- **WHEN** the Find bar is open and the user presses Enter or Shift+Enter
- **THEN** find next/previous match behavior runs and is not replaced by cursor movement

## MODIFIED Requirements

### Requirement: Tag and Message filter modal editors

The system SHALL open an in-TUI modal overlay to edit Tag and Message filter strings, activated from the filter row by mouse click or by `t` / `m` from the log viewport on the top layer. Filter values SHALL apply to log filtering in real time as the user types. Text editing in these modals SHALL support a movable insertion cursor with insert, Backspace, and Delete at the cursor per the text input insertion cursor requirement. Pressing Esc SHALL close the modal and return focus to the log viewport without reverting filter values already applied.

#### Scenario: Open tag filter modal

- **WHEN** the user clicks the Tag summary in the filter row or presses `t` while on the top layer with log viewport focus
- **THEN** a modal overlay opens with a text input for the Tag filter substring and the insertion cursor at the end of the current value

#### Scenario: Live tag filter apply

- **WHEN** the Tag filter modal is open and the user types characters
- **THEN** the Tag filter updates and filtered log output refreshes without requiring a separate confirm action

#### Scenario: Close tag filter modal

- **WHEN** the Tag filter modal is open and the user presses Esc
- **THEN** the modal closes, focus returns to the log viewport, and the current Tag filter value remains in effect

#### Scenario: Open message filter modal

- **WHEN** the user clicks the Message summary in the filter row or presses `m` while on the top layer with log viewport focus
- **THEN** a modal overlay opens with a text input for the Message filter substring and the insertion cursor at the end of the current value

#### Scenario: q types in filter modal

- **WHEN** a Tag or Message filter modal is open and the user presses `q`
- **THEN** the character is inserted at the cursor in the filter input and the application does not quit

### Requirement: Contextual mouse cursor in log shell

The system SHALL display a text (I-beam) cursor when the mouse is over the log viewport or over an active Tag filter, Message filter, or Find bar text input value region, and a default cursor when the mouse is over other toolbar, filter, or status chrome, while mouse capture is enabled.

#### Scenario: Cursor over log viewport

- **WHEN** the mouse moves over the log viewport and no modal is open
- **THEN** the terminal cursor shape is I-beam (steady bar)

#### Scenario: Cursor over text input value region

- **WHEN** the mouse moves over the Tag filter, Message filter, or Find bar input value region
- **THEN** the terminal cursor shape is I-beam

#### Scenario: Cursor over toolbar

- **WHEN** the mouse moves over the toolbar or filter row outside text input value regions
- **THEN** the terminal cursor shape is the default pointer/arrow shape
