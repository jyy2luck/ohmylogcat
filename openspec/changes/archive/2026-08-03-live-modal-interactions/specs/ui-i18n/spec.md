## MODIFIED Requirements

### Requirement: Language preference values

The system SHALL support a language preference with exactly four values: Auto, English, Simplified Chinese, and Traditional Chinese. The default preference SHALL be Auto.

#### Scenario: Default language preference

- **WHEN** settings are loaded and no language preference has been stored
- **THEN** the language preference is Auto

#### Scenario: Explicit English preference

- **WHEN** the user selects English as the language preference in Settings
- **THEN** subsequent UI chrome is rendered in English regardless of the system locale

#### Scenario: Explicit Simplified Chinese preference

- **WHEN** the user selects Simplified Chinese as the language preference in Settings
- **THEN** subsequent UI chrome is rendered in Simplified Chinese

#### Scenario: Explicit Traditional Chinese preference

- **WHEN** the user selects Traditional Chinese as the language preference in Settings
- **THEN** subsequent UI chrome is rendered in Traditional Chinese

### Requirement: Language applies on settings save

The system SHALL recompute the active UI locale from the language preference immediately when the user adjusts the Language row in the Settings modal, so chrome updates without requiring Enter save or an application restart.

#### Scenario: Immediate locale switch when adjusting language

- **WHEN** the Settings modal is open, language is focused, and the user cycles language from Auto to Traditional Chinese
- **THEN** chrome in the Settings modal and main shell updates to Traditional Chinese without pressing Enter and without restarting the process
