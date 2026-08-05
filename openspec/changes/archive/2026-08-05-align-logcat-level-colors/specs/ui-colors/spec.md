## REMOVED Requirements

### Requirement: Semantic accents use the Android Logcat 4-color palette

The system SHALL apply a single Android Logcat level palette for log level colors, identical in light- and dark-background terminals:

- **Verbose**: uncolored (terminal default foreground), with no applied tint.
- **Debug**: blue `#5F87D7`.
- **Info**: green `#00D700`.
- **Warn**: orange `#D75F00`.
- **Error / Fatal**: red `#FF0000`.

Focus highlight, text selection, and find-match highlighting SHALL use fixed interaction accents independent of the level palette. The level palette SHALL NOT depend on a user theme preference or host background detection. Shell chrome SHALL continue to inherit terminal defaults.

#### Scenario: Verbose entries use the terminal default

- **WHEN** the log viewport shows a Verbose entry
- **THEN** that entry's level styling applies no foreground tint and uses the terminal default foreground

#### Scenario: Dark terminal uses Android palette colors

- **WHEN** the application runs in a dark-background terminal
- **THEN** Debug level styling uses blue, Info uses green, Warn uses orange, and Error uses red

#### Scenario: Light terminal uses the same Android palette colors

- **WHEN** the application runs in a light-background terminal
- **THEN** Debug level styling uses blue, Info uses green, Warn uses orange, and Error uses red, identical to the dark-terminal palette

#### Scenario: Info is visually distinct from the find highlight

- **WHEN** a find match is shown in an Info-level log entry
- **THEN** the find-match highlight remains black-on-yellow and does not blend with the green Info foreground

#### Scenario: Error level remains distinct

- **WHEN** the log viewport shows an Error or Fatal entry
- **THEN** that entry's level styling uses red and remains visually distinct from Info

#### Scenario: Find and selection remain distinct

- **WHEN** find highlights and a text selection are both visible in the log viewport
- **THEN** find-match styling and selection styling remain visually distinct from each other and from unhighlighted log text

#### Scenario: No theme preference controls accents

- **WHEN** the application applies level or interaction accents
- **THEN** the selection does not read or write a user theme preference and Settings still exposes no Theme row

**Reason**: The single `logprint.c` neon palette does not match Android Studio Logcat (Info too harsh). Replaced by dual AS Logcat dark/light palettes with silent background detection.

**Migration**: Replace with "Semantic accents use dual Android Studio Logcat palettes".

## ADDED Requirements

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
