## Purpose

Defines how the TUI picks colors: shell chrome follows the host terminal defaults, while log levels and interaction accents use a single fixed semantic palette with no user-selectable theme mode.

## ADDED Requirements

### Requirement: Shell chrome follows terminal defaults

The system SHALL render main-shell chrome text (toolbar labels, filter-row labels, separators, status text, empty-state hints) without forcing a light or dark foreground palette, so those elements inherit the host terminal's default colors the same way Settings and Tag/Message modals do.

#### Scenario: Shell text readable on dark terminal

- **WHEN** the application runs in a dark-background terminal
- **THEN** toolbar, filter-row, separator, and status chrome remain readable using the terminal default colors without applying a light-theme forced black foreground

#### Scenario: Shell text readable on light terminal

- **WHEN** the application runs in a light-background terminal
- **THEN** toolbar, filter-row, separator, and status chrome remain readable using the terminal default colors without applying a dark-theme forced light-gray foreground

### Requirement: Semantic accents use a fixed palette

The system SHALL use one fixed accent palette for log level colors, focus highlight, text selection, and find-match highlighting. The palette SHALL NOT depend on a user theme preference or Auto background detection.

#### Scenario: Error level remains distinct

- **WHEN** the log viewport shows an Error or Fatal entry
- **THEN** that entry's level styling uses the fixed error accent and remains visually distinct from Info

#### Scenario: Find and selection remain distinct

- **WHEN** find highlights and a text selection are both visible in the log viewport
- **THEN** find-match styling and selection styling remain visually distinct from each other and from unhighlighted log text

### Requirement: No user-selectable theme mode

The system SHALL NOT expose Auto, Dark, or Light theme modes in Settings or elsewhere in the UI.

#### Scenario: Settings has no theme row

- **WHEN** the user opens the Settings modal
- **THEN** no Theme preference row is shown and theme cannot be cycled
