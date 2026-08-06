## MODIFIED Requirements

### Requirement: Display buffer usage in status bar

The system SHALL show in the status bar: a streaming-state indicator, filtered entry count, stored entry count, configured maximum capacity, approximate throughput, and estimated memory usage. Numeric clusters SHALL each be followed by a short parenthetical label in the active UI language. The throughput unit SHALL be localized (not hard-coded English). The streaming indicator SHALL reflect whether the logcat stream is active, using wording that means streaming/pulling logs rather than a generic "live/real-time" label.

#### Scenario: Status bar during streaming

- **WHEN** logs are streaming actively and the UI language is English
- **THEN** the status bar shows a Streaming indicator, the triple `filtered/stored/max` with parenthetical `(filtered/stored/max)`, a localized throughput unit with parenthetical `(rate)`, and a memory estimate with parenthetical `(mem)`

#### Scenario: Status bar labels follow UI language

- **WHEN** the UI language is Simplified Chinese
- **THEN** the streaming indicator uses 拉流中 (or 空闲 when idle), the count triple uses `(筛选/已存/上限)`, throughput uses `行/秒(速率)`, and memory uses `(内存)`

#### Scenario: Status bar Traditional Chinese labels

- **WHEN** the UI language is Traditional Chinese
- **THEN** the streaming indicator uses 拉流中 (or 空閒 when idle), the count triple uses `(篩選/已存/上限)`, throughput uses `行/秒(速率)`, and memory uses `(記憶體)`
