# ui-colors Specification

## Purpose

Defines how the TUI picks colors: shell chrome follows the host terminal defaults, while log levels use the single Android Logcat 4-color palette (Verbose uncolored, Debug blue, Info green, Warn orange, Error red) with no background detection and no user-selectable theme mode.

## Requirements

### Requirement: Shell chrome follows terminal defaults

The system SHALL render main-shell chrome text (toolbar labels, filter-row labels, separators, status text, empty-state hints) without forcing a light or dark foreground palette, so those elements inherit the host terminal's default colors the same way Settings and Tag/Message modals do.

#### Scenario: Shell text readable on dark terminal

- **WHEN** the application runs in a dark-background terminal
- **THEN** toolbar, filter-row, separator, and status chrome remain readable using the terminal default colors without applying a light-theme forced black foreground

#### Scenario: Shell text readable on light terminal

- **WHEN** the application runs in a light-background terminal
- **THEN** toolbar, filter-row, separator, and status chrome remain readable using the terminal default colors without applying a dark-theme forced light-gray foreground

### Requirement: Semantic accents use dual Android Studio Logcat palettes

The system SHALL apply one of two RGB level palettes from Android Studio's Android Logcat color scheme, chosen by silent host-terminal background detection:

- **Dark palette**: Verbose `#BBBBBB`, Debug `#299999`, Info `#ABC023`, Warn `#BBB529`, Error/Fatal `#FF6B68`.
- **Light palette**: Verbose `#000000`, Debug `#389FD6`, Info `#59A869`, Warn `#645607`, Error/Fatal `#CD0000`.

Focus highlight, text selection, and find-match highlighting SHALL use fixed interaction accents independent of which level palette is active. The choice SHALL NOT depend on a user theme preference. Shell chrome SHALL continue to inherit terminal defaults independently of which accent palette is active.

#### Scenario: Light terminal uses light Info green

- **WHEN** the host terminal background is detected as light
- **THEN** Info-level log styling uses the light accent Info color (`#59A869`) and remains visually distinct from Error

#### Scenario: Dark terminal uses dark Info olive

- **WHEN** the host terminal background is detected as dark
- **THEN** Info-level log styling uses the dark accent Info color (`#ABC023`) and remains visually distinct from Error

#### Scenario: Detection failure falls back to dark

- **WHEN** background detection fails or is inconclusive
- **THEN** the dark accent palette is used

#### Scenario: Error level remains distinct

- **WHEN** the log viewport shows an Error or Fatal entry
- **THEN** that entry's level styling uses the active palette's error accent and remains visually distinct from Info

#### Scenario: Info is visually distinct from the find highlight

- **WHEN** a find match is shown in an Info-level log entry
- **THEN** the find-match highlight remains black-on-yellow and does not blend with the Info foreground

#### Scenario: Find and selection remain distinct

- **WHEN** find highlights and a text selection are both visible in the log viewport
- **THEN** find-match styling and selection styling remain visually distinct from each other and from unhighlighted log text

#### Scenario: No theme preference controls accents

- **WHEN** the application selects an accent palette
- **THEN** the selection does not read or write a user theme preference and Settings still exposes no Theme row

### Requirement: No user-selectable theme mode

The system SHALL NOT expose Auto, Dark, or Light theme modes in Settings or elsewhere in the UI.

#### Scenario: Settings has no theme row

- **WHEN** the user opens the Settings modal
- **THEN** no Theme preference row is shown and theme cannot be cycled
