## MODIFIED Requirements

### Requirement: Post-update version check without GitHub REST API

After an immediate successful update install, when the system compares the running package version to the latest GitHub Release version, it SHALL resolve that latest version without requiring unauthenticated access to the GitHub Releases REST API (for example by following the public `releases/latest` redirect to a tag URL). On Windows, when that public redirect is reachable, that discovery SHALL succeed in resolving a parseable release version; treating the first redirect hop as a hard client error (for example a maximum-redirection-exceeded failure from a no-follow request) SHALL NOT by itself send the command into unverified messaging. If version discovery fails for other reasons, the system MAY still report that the install script finished and suggest verifying with `--version`, consistent with existing unverified messaging.

#### Scenario: Version comparison when REST API is rate-limited

- **WHEN** the user runs `ohmylogcat update`, the install script completes an immediate install, and the GitHub Releases REST API is rate-limited
- **THEN** the process still reports already-up-to-date or updated-toward-latest based on a non-API latest-version discovery when that discovery succeeds

#### Scenario: Version discovery failure remains non-fatal

- **WHEN** the install script completes successfully but latest-version discovery fails
- **THEN** the process does not treat the update as failed solely for that reason and informs the user how to confirm the installed version

#### Scenario: Windows verifies latest version when GitHub is reachable

- **WHEN** the user runs `ohmylogcat update` on Windows, the install script completes an immediate install, and the public `releases/latest` redirect is reachable
- **THEN** the process reports already-up-to-date or updated-toward-latest including the discovered version, and does not emit the unverified-discovery warning solely because a no-follow request hit a redirect limit
