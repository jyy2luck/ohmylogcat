## MODIFIED Requirements

### Requirement: Display buffer usage in status bar

The system SHALL show in the status bar: a session-state indicator shared with the toolbar (streaming / paused / disconnected), filtered entry count, and stored entry count. The status bar SHALL NOT show focus-target, wrap-state, throughput, memory, or parenthetical legends. Errors and ephemeral copy/export messages SHALL occupy the middle of the status row, between the count cluster and the version cluster, rather than appending to the count cluster or replacing the version cluster. The streaming indicator SHALL use wording that means streaming/pulling logs rather than a generic "live/real-time" label, and SHALL match the toolbar session label.

#### Scenario: Status bar during streaming

- **WHEN** logs are streaming actively, display is not paused, and the UI language is English
- **THEN** both the toolbar and the status bar show a Streaming session label and the status bar shows `filtered / stored` without a max/rate/memory legend

#### Scenario: Status bar labels follow UI language

- **WHEN** the UI language is Simplified Chinese
- **THEN** the session indicator uses 拉流中, 已暂停, or 已断开 (matching the toolbar), and the count pair is `筛选数 / 已存` without parenthetical legends

#### Scenario: Status bar Traditional Chinese labels

- **WHEN** the UI language is Traditional Chinese
- **THEN** the session indicator uses 拉流中, 已暫停, or 已斷開 (matching the toolbar)

#### Scenario: Paused agrees across chrome

- **WHEN** streaming is paused
- **THEN** the toolbar session label and the status bar session label both read as paused (not Streaming / 拉流中)

#### Scenario: Error does not replace version cluster

- **WHEN** the status bar is showing the version cluster and an error or copy/export message is also present
- **THEN** the message appears between the count cluster and the version cluster, and the version cluster remains at the far right of the status row

## ADDED Requirements

### Requirement: Status bar version cluster

The system SHALL pin a version cluster at the far right of the status bar. The cluster SHALL always include the running package version. When a newer GitHub Release version is known, the cluster SHALL append a localized update suffix after the current version with no space before the opening parenthesis. Copy SHALL follow the active UI locale:

- English: `v{current}` and `v{current} (update {latest})`
- Simplified Chinese: `版本-{current}` and `版本-{current}（有更新{latest}）`
- Traditional Chinese: `版本-{current}` and `版本-{current}（有更新{latest}）`

`{current}` and `{latest}` SHALL be the numeric version without a leading `v`. Until a newer remote version is known, including while the check is in flight and after a failed check, the cluster SHALL show only the current version. When the status row is too narrow to hold counts, optional middle message, and the full cluster, the system SHALL drop the update suffix before dropping the current-version text, and SHALL drop or truncate the middle message before dropping the current-version text.

#### Scenario: Current version on launch

- **WHEN** the TUI status bar is shown and no newer remote version is known yet
- **THEN** the far right of the status bar shows the localized current-version text and does not show an update suffix

#### Scenario: Update suffix in Simplified Chinese

- **WHEN** the active UI locale is Simplified Chinese, the running package version is `0.6.0`, and a newer GitHub Release version `0.7.0` is known
- **THEN** the far right of the status bar shows `版本-0.6.0（有更新0.7.0）`

#### Scenario: Update suffix in English

- **WHEN** the active UI locale is English, the running package version is `0.6.0`, and a newer GitHub Release version `0.7.0` is known
- **THEN** the far right of the status bar shows `v0.6.0 (update 0.7.0)`

#### Scenario: Update suffix in Traditional Chinese

- **WHEN** the active UI locale is Traditional Chinese, the running package version is `0.6.0`, and a newer GitHub Release version `0.7.0` is known
- **THEN** the far right of the status bar shows `版本-0.6.0（有更新0.7.0）`

#### Scenario: Narrow width drops update suffix first

- **WHEN** a newer remote version is known and the status row cannot fit the count cluster plus the full version cluster
- **THEN** the update suffix is omitted and the current-version text remains visible
