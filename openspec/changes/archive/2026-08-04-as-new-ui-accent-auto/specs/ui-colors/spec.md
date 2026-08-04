## REMOVED Requirements

### Requirement: Semantic accents use a fixed palette

**Reason**: A single ANSI palette (Info as Gray) is unreadable on light terminals and does not match Android Studio Logcat colors for light vs dark hosts.

**Migration**: Use the new dual Android Studio–aligned accent palettes with silent background detection (dark fallback).

## ADDED Requirements

### Requirement: Semantic accents use dual Android Studio palettes

The system SHALL apply one of two RGB accent palettes for log level colors, focus highlight, text selection, and find-match highlighting, chosen by silent host-terminal background detection:

- **Light palette**: IntelliJ Default / IntelliJ Light console `LOG_*` colors (Info green).
- **Dark palette**: Android Studio New UI / Islands Dark console `LOG_*` colors (Info gold).

The choice SHALL NOT depend on a user theme preference. Shell chrome SHALL continue to inherit terminal defaults independently of which accent palette is active.

#### Scenario: Light terminal uses light Info green

- **WHEN** the host terminal background is detected as light
- **THEN** Info-level log styling uses the light accent Info color (green) and remains visually distinct from Error

#### Scenario: Dark terminal uses New UI Info gold

- **WHEN** the host terminal background is detected as dark
- **THEN** Info-level log styling uses the dark accent Info color (gold) and remains visually distinct from Error

#### Scenario: Detection failure falls back to dark

- **WHEN** background detection fails or is inconclusive
- **THEN** the dark accent palette is used

#### Scenario: Error level remains distinct

- **WHEN** the log viewport shows an Error or Fatal entry
- **THEN** that entry's level styling uses the active palette's error accent and remains visually distinct from Info

#### Scenario: Find and selection remain distinct

- **WHEN** find highlights and a text selection are both visible in the log viewport
- **THEN** find-match styling and selection styling remain visually distinct from each other and from unhighlighted log text

#### Scenario: No theme preference controls accents

- **WHEN** the application selects an accent palette
- **THEN** the selection does not read or write a user theme preference and Settings still exposes no Theme row
