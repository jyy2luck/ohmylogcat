# ui-i18n Specification

## Purpose

Provides UI locale preference, Auto resolution from the system locale, and a localized string catalog so the TUI shell can render English, Simplified Chinese, or Traditional Chinese chrome.

## Requirements

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

### Requirement: Auto resolves from system locale

When the language preference is Auto, the system SHALL resolve the active UI locale from the host system locale using these rules: locales indicating Simplified Chinese script or region (`Hans`, `CN`, `SG`) map to Simplified Chinese; locales indicating Traditional Chinese script or region (`Hant`, `TW`, `HK`, `MO`) map to Traditional Chinese; a bare or otherwise ambiguous Chinese locale (`zh` without Hans/Hant or a recognized region) maps to Simplified Chinese; English locales (`en` and variants) map to English; any other or unrecognized locale maps to English.

#### Scenario: Auto with Simplified Chinese system locale

- **WHEN** language preference is Auto and the system locale indicates Simplified Chinese (for example `zh-CN` or `zh-Hans`)
- **THEN** the active UI locale is Simplified Chinese

#### Scenario: Auto with Traditional Chinese system locale

- **WHEN** language preference is Auto and the system locale indicates Traditional Chinese (for example `zh-TW`, `zh-HK`, or `zh-Hant`)
- **THEN** the active UI locale is Traditional Chinese

#### Scenario: Auto with bare zh system locale

- **WHEN** language preference is Auto and the system locale is bare `zh` without Hans/Hant or a recognized region tag
- **THEN** the active UI locale is Simplified Chinese

#### Scenario: Auto with English system locale

- **WHEN** language preference is Auto and the system locale indicates English
- **THEN** the active UI locale is English

#### Scenario: Auto with unsupported system locale

- **WHEN** language preference is Auto and the system locale is neither Chinese nor English (for example `ja-JP` or `de-DE`)
- **THEN** the active UI locale falls back to English

### Requirement: Localized TUI chrome catalog

The system SHALL render user-facing TUI chrome strings (toolbar labels, filter labels, modal titles and help lines, status messages, and empty-state hints) from a locale-specific catalog for English, Simplified Chinese, and Traditional Chinese. Log line content originating from the device SHALL NOT be translated. Keyboard shortcut letters shown in chrome (for example `[s]`, `[q]`) SHALL remain Latin characters across locales.

#### Scenario: Toolbar label follows active locale

- **WHEN** the active UI locale is Simplified Chinese
- **THEN** toolbar action labels are shown in Simplified Chinese while shortcut letters remain Latin

#### Scenario: Status message follows active locale

- **WHEN** the user saves settings successfully and the active UI locale is Traditional Chinese
- **THEN** the success status message is shown in Traditional Chinese

#### Scenario: Log content remains untranslated

- **WHEN** log lines are displayed in the viewport
- **THEN** their message text is shown as received from the device without locale translation

### Requirement: Language applies on settings save

The system SHALL recompute the active UI locale from the language preference immediately when the user adjusts the Language row in the Settings modal, so chrome updates without requiring Enter save or an application restart.

#### Scenario: Immediate locale switch when adjusting language

- **WHEN** the Settings modal is open, language is focused, and the user cycles language from Auto to Traditional Chinese
- **THEN** chrome in the Settings modal and main shell updates to Traditional Chinese without pressing Enter and without restarting the process
