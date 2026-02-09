# Proposal: Customizable UI Hotkeys

## Summary
Implements a comprehensive system for customizable keyboard shortcuts (hotkeys) within the application UI. This allows users to remap existing hardcoded matching (like `Cmd+,` for Settings) and extends support for navigation and action shortcuts.

## Motivation
Currently, hotkeys like `Cmd+,` and `Escape` are hardcoded in the frontend. Users cannot customize them to fit their workflow or keyboard layout. Moving these to a configuration system aligns with the "comprehensive customization" requirement.

## Proposed Solution
1.  **Backend**: Update `HotkeySettings` struct to store a `ui_shortcuts` map (ActionID -> ShortcutString).
2.  **Frontend**: 
    - Expose these settings in `SettingsPanel.vue` with a new "Shortcuts" section.
    - Implement a `useShortcut` composable or utility to handle key binding matching against the config.
    - Replace hardcoded `e.key === ...` checks with this system.
