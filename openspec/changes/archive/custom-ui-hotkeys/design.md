# Design: Customizable UI Hotkeys

## Data Model

### Rust (`src-tauri/src/models/settings.rs`)

Update `HotkeySettings` to include `ui_shortcuts`.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeySettings {
    pub global_shortcut: String,
    // Map of Action ID -> Shortcut Key Combination (e.g. "general.open_settings" -> "Cmd+,")
    #[serde(default = "default_ui_shortcuts")]
    pub ui_shortcuts: std::collections::HashMap<String, String>,
}
```

### Types (`src/types/settings.ts`)

```typescript
export interface HotkeySettings {
  global_shortcut: string;
  ui_shortcuts: Record<string, string>;
}
```

## Action Registry

We will define a set of Action IDs that the system supports.

| Action ID | Default (Mac) | Default (Win/Lin) | Description |
|-----------|---------------|-------------------|-------------|
| `general.close` | `Escape` | `Escape` | Close the launcher window |
| `general.settings` | `Meta+,` | `Ctrl+,` | Open Settings panel |
| `search.next` | `ArrowDown` | `ArrowDown` | Select next result |
| `search.prev` | `ArrowUp` | `ArrowUp` | Select previous result |
| `search.open` | `Enter` | `Enter` | Open selected result |
| `search.open_new_tab` | `Meta+Enter` | `Ctrl+Enter` | (Future) Open in background |

## UI UX

- **Settings Panel**: Add a "Shortcuts" section.
- **Editor**: A list of actions with their current keybinding. Clicking a binding enters "recording" mode where the user presses the desired combination.
- **Validation**: Check for conflicts. If user tries to bind `ArrowDown` to `general.close`, warn them.

## Implementation Details

- **Event Handling**: 
  - Create a `ShortcutManager` class or Composable in Vue.
  - It loads settings.
  - It provides a `matches(event, actionId)` function.
  - `App.vue` and `SearchCombobox.vue` will use this to check key events.

```typescript
// Example usage
if (shortcutManager.matches(event, 'general.settings')) {
  toggleSettings();
}
```
