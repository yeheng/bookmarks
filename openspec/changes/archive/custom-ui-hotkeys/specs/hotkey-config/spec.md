# Hotkey Configuration Spec

## ADDED Requirements

### Requirement: Store UI Shortcuts in Settings
The system MUST persist user-defined keybindings for UI actions in the application settings.

#### Scenario: User restarts application
Custom bindings (e.g., `Alt+Q` for Close) MUST be preserved.

#### Scenario: Defaults population
Defaults MUST be populated if no settings exist (e.g., `Escape` for Close).

### Requirement: Customizable Application Shortcuts
Users MUST be able to customize shortcuts for global application actions.

#### Scenario: User changes binding
User changes "Open Settings" from `Cmd+,` to `Cmd+Shift+P`.

#### Scenario: User uses new binding
User presses `Cmd+Shift+P` -> Settings Panel opens.

#### Scenario: User uses old binding
User presses old `Cmd+,` -> Nothing happens.

### Requirement: Shortcut Recording UI
The Settings UI MUST provide a visual interface to record new shortcuts.

#### Scenario: Recording process
- User clicks "Record Shortcut" button.
- UI shows "Press keys...".
- User presses `Ctrl` then `M`. UI displays `Ctrl+M` and saves.

### Requirement: Conflict Detection (Basic)
The system MUST prevent assigning the same shortcut to two different active actions.

#### Scenario: Conflict warning
User tries to bind `ArrowDown` to "Open Settings". UI shows warning or rejects usage (since `ArrowDown` is navigation).
