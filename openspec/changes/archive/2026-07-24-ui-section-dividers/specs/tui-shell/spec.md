## ADDED Requirements

### Requirement: Visual section dividers

The system SHALL render full-width horizontal divider lines between the toolbar, filter row, log viewport, and status bar so the four main layout zones are visually distinct.

#### Scenario: Dividers visible on launch

- **WHEN** the TUI main shell is shown after launch
- **THEN** a horizontal divider appears between the toolbar and filter row, between the filter row and log viewport, and between the log viewport and status bar

#### Scenario: Dividers do not intercept input

- **WHEN** the user clicks or moves the mouse over a divider row
- **THEN** the divider does not define a separate hit target and mouse actions fall through to existing toolbar, filter, or log behaviors according to adjacent regions

### Requirement: Filter row shortcut labels

The system SHALL label Tag, Message, and Level controls in the filter row with inline keyboard shortcut hints matching toolbar style: `[t]Tag[value]`, `[m]Message[value]`, and `[l]Level[value]`. When Tag or Message filter is unset, the value brackets SHALL be empty (e.g. `[t]Tag[]`). Level SHALL always display the current minimum level name (default Verbose). The filter row SHALL NOT show a trailing `(t/m edit · l level · …)` hint; it MAY show `(click Tag/Message)` for mouse users.

#### Scenario: Empty filter labels

- **WHEN** no Tag or Message filter is active
- **THEN** the filter row shows `[t]Tag[]` and `[m]Message[]`

#### Scenario: Active filter labels

- **WHEN** Tag filter is `myapp` and Message filter is `error`
- **THEN** the filter row shows `[t]Tag[myapp]` and `[m]Message[error]`

#### Scenario: Level label always present

- **WHEN** the minimum log level is Warn
- **THEN** the filter row shows `[l]Level[Warn]`

#### Scenario: Shortcut hints bold

- **WHEN** the filter row is rendered
- **THEN** the `[t]`, `[m]`, and `[l]` shortcut segments are visually emphasized consistent with toolbar shortcut styling

#### Scenario: Mouse hint only

- **WHEN** the filter row is rendered on the main shell
- **THEN** the row does not include `(t/m edit · l level · …)` and may include `(click Tag/Message)`

## MODIFIED Requirements

### Requirement: Main layout hosts core logcat surfaces

The system SHALL present within the terminal: a top toolbar of primary actions, filter controls showing Tag and Message filter summaries plus Level control, a scrollable log viewport, and a status bar showing buffer usage. Horizontal dividers SHALL separate the toolbar, filter row, log viewport, and status bar.

#### Scenario: Initial layout visible

- **WHEN** the TUI is shown after launch
- **THEN** toolbar, filter area (including Tag and Message summaries), log viewport, and status bar are all visible and usable, with dividers between each major zone
