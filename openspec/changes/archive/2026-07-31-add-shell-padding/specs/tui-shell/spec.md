## ADDED Requirements

### Requirement: Shell viewport inset

The system SHALL render the main shell (toolbar, filter row, section dividers, log viewport, and status bar) inside a uniform inset from the terminal edges rather than flush against the terminal border. When the terminal is large enough, the inset SHALL be at least one column on the left and right and one row on the top and bottom. When the terminal is too small to preserve a usable log viewport, the system MAY reduce or disable the inset. Modal overlays SHALL continue to use the full terminal area for centering and sizing. Mouse hit regions and log viewport dimensions SHALL match the inset content area so text selection and click targets remain accurate.

#### Scenario: Inset visible on launch

- **WHEN** the TUI main shell is shown in a terminal with sufficient width and height
- **THEN** the toolbar, filter row, log viewport, and status bar are rendered inset from all four terminal edges with visible empty gutter space around the shell

#### Scenario: Inset disabled on very small terminal

- **WHEN** the terminal width or height falls below the minimum threshold for a usable inset
- **THEN** the main shell MAY render edge-to-edge without inset while remaining usable

#### Scenario: Log selection respects inset log area

- **WHEN** the user drags to select text in the log viewport
- **THEN** selection mapping uses the inset log viewport bounds and does not treat gutter cells outside the inset as log content

#### Scenario: Modals ignore shell inset

- **WHEN** a modal overlay is open
- **THEN** the modal is centered and sized relative to the full terminal area, not the inset shell content area

#### Scenario: Mouse in gutter does not start log selection

- **WHEN** the user clicks in the gutter outside the inset log viewport
- **THEN** log text selection does not start and existing chrome click or clear-selection behavior applies as for clicks outside the log viewport

## MODIFIED Requirements

### Requirement: Main layout hosts core logcat surfaces

The system SHALL present within the terminal: a top toolbar of primary actions, filter controls showing Tag and Message filter summaries plus Level control, a scrollable log viewport, and a status bar showing buffer usage. These surfaces SHALL be laid out inside the shell viewport inset when inset is active. Horizontal dividers SHALL separate the toolbar, filter row, log viewport, and status bar.

#### Scenario: Initial layout visible

- **WHEN** the TUI is shown after launch
- **THEN** toolbar, filter area (including Tag and Message summaries), log viewport, and status bar are all visible and usable inside the shell content area, with dividers between each major zone

### Requirement: Visual section dividers

The system SHALL render horizontal divider lines between the toolbar, filter row, log viewport, and status bar so the four main layout zones are visually distinct. Divider lines SHALL span the width of the shell content area (inset width when inset is active), not the full terminal width when inset is active.

#### Scenario: Dividers visible on launch

- **WHEN** the TUI main shell is shown after launch with shell inset active
- **THEN** a horizontal divider appears between the toolbar and filter row, between the filter row and log viewport, and between the log viewport and status bar, each aligned with the inset content width

#### Scenario: Dividers do not intercept input

- **WHEN** the user clicks or moves the mouse over a divider row
- **THEN** the divider does not define a separate hit target and mouse actions fall through to existing toolbar, filter, or log behaviors according to adjacent regions
