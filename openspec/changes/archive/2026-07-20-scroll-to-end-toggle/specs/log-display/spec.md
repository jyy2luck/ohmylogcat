## MODIFIED Requirements

### Requirement: Auto-scroll to end on new logs

The system SHALL automatically scroll to the newest log entry when tail-following mode is enabled and the displayed log set changes, including incremental appends and full list refreshes from filter or device changes.

#### Scenario: Auto-scroll when tail-following is on

- **WHEN** tail-following is enabled and new logs arrive via streaming append
- **THEN** the view scrolls to show the newest entry

#### Scenario: Auto-scroll after filter refresh

- **WHEN** tail-following is enabled and the visible log list is replaced due to a filter change
- **THEN** the view scrolls to show the newest visible entry

#### Scenario: Auto-scroll after device switch

- **WHEN** tail-following is enabled and logs begin arriving for a newly selected device
- **THEN** the view scrolls to show the newest entry as logs appear

#### Scenario: No auto-scroll when tail-following is off

- **WHEN** tail-following is disabled and new logs arrive or the list refreshes
- **THEN** the scroll position is not forced to the newest entry

#### Scenario: No auto-scroll when scrolled up

- **WHEN** the user scrolls up away from the bottom while tail-following is enabled
- **THEN** tail-following is disabled and new logs do not force the scroll position to jump

#### Scenario: Find suspends auto-scroll

- **WHEN** the in-view find bar is active with one or more matches
- **THEN** tail-following scroll behavior is suspended until find is closed

### Requirement: Scroll to end control

The system SHALL provide a Scroll to End toolbar control that acts as a persistent tail-following toggle aligned with Android Studio Logcat, with visual on/off state and preference persistence across application restarts.

#### Scenario: Default tail-following on first launch

- **WHEN** the user opens the application for the first time or has no saved preference
- **THEN** tail-following is enabled by default

#### Scenario: Persist tail-following preference

- **WHEN** the user toggles Scroll to End
- **THEN** the tail-following preference is persisted and restored on the next application launch

#### Scenario: Enable tail-following from toolbar

- **WHEN** tail-following is off and the user clicks Scroll to End
- **THEN** tail-following turns on and the view jumps to the newest entry

#### Scenario: Disable tail-following from toolbar

- **WHEN** tail-following is on and the user clicks Scroll to End
- **THEN** tail-following turns off

#### Scenario: Tail-following survives list reset events

- **WHEN** tail-following is on and the user clears logs or switches devices
- **THEN** tail-following remains enabled without requiring the user to click Scroll to End again
