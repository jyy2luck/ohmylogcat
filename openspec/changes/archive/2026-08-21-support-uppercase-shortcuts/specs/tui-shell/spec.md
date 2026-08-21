## ADDED Requirements

### Requirement: Case-insensitive letter action shortcuts

The system SHALL treat the lowercase and uppercase forms of each letter as equivalent when activating a letter shortcut in the terminal user interface, including shortcuts entered with `Shift` or while `Caps Lock` is enabled. This rule SHALL apply to the main shell and modal overlays, except for the intentionally distinct `n` and `N` find-navigation shortcuts. Existing modifier-key restrictions and non-letter shortcuts SHALL remain unchanged.

#### Scenario: Uppercase main-shell shortcut

- **WHEN** the log viewport has focus and the user presses the uppercase form of a letter shortcut, such as `C` for Clear or `S` for Settings
- **THEN** the same action runs as for the corresponding lowercase shortcut

#### Scenario: Uppercase modal shortcut

- **WHEN** a modal is open and the user presses the uppercase form of one of that modal's letter shortcuts, such as `R` to refresh devices or `A` to export all buffered logs
- **THEN** the same modal action runs as for the corresponding lowercase shortcut

#### Scenario: Modifier restrictions remain effective

- **WHEN** a letter shortcut has an existing restriction on `Ctrl`, `Cmd`, or another modifier and the user presses its uppercase form with that restricted modifier
- **THEN** the shortcut is accepted or ignored according to the same restriction as the lowercase form

#### Scenario: Uppercase text remains editable

- **WHEN** a text-input field is focused and the user types an uppercase letter that is also a shortcut elsewhere
- **THEN** the uppercase character is inserted into the field unchanged and does not trigger the unrelated shortcut action

#### Scenario: Non-letter shortcut behavior is unchanged

- **WHEN** the user presses a non-letter shortcut such as Space, `/`, an arrow key, or Enter
- **THEN** the existing behavior for that shortcut remains unchanged
