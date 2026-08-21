## ADDED Requirements

### Requirement: Case-insensitive log-view letter shortcuts

The system SHALL treat the lowercase and uppercase forms of each letter shortcut used by the log viewport, Find bar, and selection actions as equivalent, including shortcuts entered with `Shift` or while `Caps Lock` is enabled. The existing `n` and `N` find-navigation shortcuts SHALL remain case-sensitive and retain their separate next-match and previous-match actions. Existing `Ctrl`/`Cmd` modifier semantics SHALL remain unchanged.

#### Scenario: Uppercase Find activation

- **WHEN** no modal is open and the user presses the Find shortcut with an uppercase `F` character, including `Ctrl+F` or `Cmd+F` when delivered by the terminal
- **THEN** the Find bar receives focus and follows the same open or select-query behavior as the lowercase `f` form

#### Scenario: Uppercase selection copy

- **WHEN** a non-empty log selection exists and the user presses `Ctrl+C` or `Cmd+C` with an uppercase `C` character
- **THEN** the selected text is copied exactly as it is for the lowercase `c` character

#### Scenario: Next-match shortcut preserves its case pair

- **WHEN** the Find bar is open and focused and the user presses lowercase `n`
- **THEN** the view navigates to the next match

#### Scenario: Previous-match shortcut preserves its case pair

- **WHEN** the Find bar is open and focused and the user presses uppercase `N`
- **THEN** the view navigates to the previous match

#### Scenario: Uppercase log action

- **WHEN** the log viewport has focus and the user presses the uppercase form of a letter action shortcut, such as `F` for Follow or `W` for Soft-Wrap
- **THEN** the same log action runs as for the corresponding lowercase shortcut
