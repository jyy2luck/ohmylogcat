## ADDED Requirements

### Requirement: Non-blocking latest-release check in the TUI

When the TUI starts, the system SHALL resolve the latest GitHub Release version without blocking the first paint or log streaming. Discovery SHALL use the public `releases/latest` redirect to a tag URL (or an equivalent non-REST path) and SHALL NOT require unauthenticated access to the GitHub Releases REST API. The system SHALL treat the remote version as newer only when its numeric major.minor.patch triple is greater than the running package version's numeric triple (leading `v` stripped; a local pre-release suffix such as `-dev` is ignored for the triple). The TUI SHALL NOT download a replacement binary, invoke the install script, or otherwise apply an update from inside the TUI. A failed, timed-out, or unparsable discovery SHALL leave the version cluster on current version only and SHALL NOT write a status-bar error for that failure.

#### Scenario: First paint does not wait for GitHub

- **WHEN** the user launches the TUI
- **THEN** the main shell including the current-version cluster is shown without waiting for latest-release discovery to finish

#### Scenario: Newer release appends the update suffix

- **WHEN** latest-release discovery succeeds with a version whose numeric triple is greater than the running package version
- **THEN** the status bar version cluster gains the localized update suffix for that remote version

#### Scenario: Equal or older remote stays current-only

- **WHEN** latest-release discovery succeeds with a version whose numeric triple is less than or equal to the running package version
- **THEN** the version cluster does not show an update suffix

#### Scenario: Discovery failure is silent

- **WHEN** latest-release discovery fails because the network is unavailable, the request times out, or the redirect cannot be parsed as a release tag
- **THEN** the version cluster continues to show only the current version and the status-bar error slot is not set solely because of that failure

#### Scenario: TUI does not self-update

- **WHEN** a newer GitHub Release is known while the TUI is running
- **THEN** the process does not replace its own binary or run the platform install script as a result of that knowledge
