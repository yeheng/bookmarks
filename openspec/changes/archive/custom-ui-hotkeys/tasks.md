# Tasks: Customizable UI Hotkeys

- [x] Backend: Update `HotkeySettings` struct in Rust @src-tauri/src/models/settings.rs
    - Add `ui_shortcuts` HashMap.
    - Implement defaults in `default_ui_shortcuts`.
- [x] Frontend: Update `HotkeySettings` interface @launcher-app/src/types/settings.ts
- [x] Frontend: Create `ShortcutManager` utility/store @launcher-app/src/services/shortcuts.ts
    - Parse shortcut strings (e.g. "Meta+,").
    - Match keyboard events.
- [x] Frontend: Refactor `App.vue` to use `ShortcutManager` for "Settings" and "Close"
- [x] Frontend: Refactor `SearchCombobox.vue` to potentially use `ShortcutManager` (investigate Headless UI override)
- [x] Frontend: Update `SettingsPanel.vue` to include Shortcut Editor UI
    - Component for recording keystrokes.
    - List of actions.
- [x] Verification: Test all default shortcuts working manually.
- [x] Verification: Test customizing a shortcut (e.g. change settings to `Ctrl+Shift+S`) and verify it works.
