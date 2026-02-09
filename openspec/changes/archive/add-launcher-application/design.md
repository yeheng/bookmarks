# Design Document: Desktop Launcher Application

## Context

Building a desktop productivity launcher that provides instant access to bookmarks and files through a global hotkey. The application must be fast (<100ms search), minimal, and cross-platform. Users are familiar with tools like Raycast, Alfred, and Spotlight, so the UX should feel natural to those coming from those ecosystems.

**Constraints:**

- Performance: Search must respond in <100ms
- Cross-platform: Must work on macOS, Windows, and Linux
- Resource efficiency: Minimal memory footprint when idle
- UX: Simple, keyboard-driven, minimal UI

**Stakeholders:**

- End users: Power users seeking productivity improvements
- Developers: Maintainers of the codebase

## Goals / Non-Goals

**Goals:**

- Build a fast, responsive launcher with global hotkey access
- Support bookmark management (browser import + manual)
- Support file search across user-defined directories
- Create a minimalist, keyboard-first UI
- Maintain cross-platform compatibility

**Non-Goals:**

- Plugin system or extensibility (v1)
- Cloud sync capabilities (v1)
- Web search or calculator features (v1)
- Advanced file preview capabilities (v1)

## Decisions

### 1. Technology Stack

**Decision:** Tauri 2.x + Vue.js 3 + TypeScript + HeadlessUI/Vue

**Why:**

- **Tauri**: Lightweight, secure, cross-platform, native performance, small bundle size (~3-5MB vs Electron's ~100MB+)
- **Vue.js 3**: Reactive, performant, good TypeScript support, familiar ecosystem
- **HeadlessUI/Vue**: Accessible, unstyled components perfect for custom minimal design
- **TypeScript**: Type safety, better DX, easier maintenance

**Alternatives considered:**

- Electron: Rejected due to large bundle size and memory overhead
- React: Rejected in favor of Vue.js due to user preference
- Native (Swift/C#/Qt): Rejected due to cross-platform maintenance complexity

### 2. Data Storage

**Decision:** SQLite with FTS5 (Full-Text Search) for both bookmarks and file index

**Why:**

- Local-first, no network dependencies
- FTS5 provides fast full-text search with ranking
- ACID compliance for data integrity
- Cross-platform, embedded, zero-config
- Proven performance for <1M records

**Schema:**

```sql
-- Bookmarks table
CREATE TABLE bookmarks (
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  url TEXT NOT NULL UNIQUE,
  description TEXT,
  favicon_url TEXT,
  tags TEXT, -- JSON array
  source TEXT, -- 'chrome', 'firefox', 'safari', 'manual'
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  last_accessed INTEGER
);

-- FTS5 virtual table for bookmark search
CREATE VIRTUAL TABLE bookmarks_fts USING fts5(
  title, url, description, tags,
  content=bookmarks,
  content_rowid=id
);

-- File index table
CREATE TABLE files (
  id INTEGER PRIMARY KEY,
  path TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  extension TEXT,
  size INTEGER,
  modified_at INTEGER,
  created_at INTEGER,
  indexed_at INTEGER NOT NULL
);

-- FTS5 virtual table for file search
CREATE VIRTUAL TABLE files_fts USING fts5(
  name, path,
  content=files,
  content_rowid=id
);

-- Settings table
CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

-- Usage history for ranking
CREATE TABLE usage_history (
  id INTEGER PRIMARY KEY,
  resource_type TEXT NOT NULL, -- 'bookmark' or 'file'
  resource_id INTEGER NOT NULL,
  accessed_at INTEGER NOT NULL
);
```

**Alternatives considered:**

- In-memory search: Rejected due to cold-start performance and memory usage
- Tantivy (Rust search engine): Rejected as overkill for this scale
- JSON files: Rejected due to poor search performance

### 3. Search Algorithm

**Decision:** Hybrid approach combining FTS5 ranking + frecency scoring

**Algorithm:**

1. FTS5 full-text search provides initial matches with BM25 ranking
2. Apply frecency boost (frequency + recency) based on usage_history
3. Sort by combined score
4. Return top 10 results

**Frecency formula:**

```
score = fts_score * 0.7 + frecency_score * 0.3
frecency_score = (frequency * 0.6) + (recency * 0.4)
```

**Why:**

- FTS5 provides excellent text matching
- Frecency learns user behavior over time
- Combined approach balances relevance and personalization
- <100ms performance achievable with proper indexing

**Alternatives considered:**

- Pure fuzzy search (fzf-style): Rejected due to SQLite FTS5 being faster for our use case
- Pure frecency: Rejected as it doesn't handle new items well
- Machine learning ranking: Rejected as overkill for v1

### 4. Global Hotkey Implementation

**Decision:** Platform-specific implementations via Tauri plugins

**Implementation:**

- macOS: Use `global-hotkey` crate with Cocoa APIs
- Windows: Use `global-hotkey` crate with Win32 APIs
- Linux: Use `global-hotkey` crate with X11/Wayland

**Default hotkey:** Cmd+Space (macOS), Ctrl+Space (Windows/Linux)

**Why:**

- Native hotkey registration ensures reliability
- Platform-specific approach provides best UX
- Tauri plugins abstract platform differences

### 5. Window Behavior

**Decision:** Frameless, transparent window with auto-hide

**Specifications:**

- **Size:** 680px wide × 480px tall (max height with results)
- **Position:** Center of screen (configurable)
- **Appearance:**
  - Frameless (no title bar)
  - Transparent background with blur (macOS/Windows)
  - Rounded corners (12px radius)
  - Shadow for depth
- **Behavior:**
  - Always on top when visible
  - Hide on Esc or focus loss
  - Hide on result selection
  - Smooth fade in/out (150ms)

**Why:**

- Frameless = minimal, modern aesthetic
- Auto-hide = non-intrusive, quick access pattern
- Transparency + blur = visual polish without being distracting

### 6. File Indexing Strategy

**Decision:** Incremental indexing with file system watchers

**Initial scan:**

1. User configures search directories
2. Background thread scans directories recursively
3. Store file metadata in SQLite
4. Build FTS5 index
5. Report progress to UI

**Incremental updates:**

1. Use file system watchers (notify crate in Rust)
2. Detect file create/modify/delete events
3. Update index in real-time
4. Debounce events (100ms) to avoid thrashing

**Performance optimizations:**

- Skip hidden files and system directories by default
- Limit to 100,000 files initially (configurable)
- Index only file names and paths, not content (v1)
- Use prepared statements for batch inserts

**Why:**

- Initial scan handles existing files
- Watchers keep index fresh without polling
- Incremental approach scales better than full re-scans

**Alternatives considered:**

- Full re-scan on startup: Rejected due to slow startup time
- Content indexing: Deferred to v2 due to complexity
- External indexing tools (mdfind/Everything): Rejected to maintain cross-platform consistency

### 7. Bookmark Import

**Decision:** Parse browser bookmark files directly

**Browser locations:**

- Chrome: `~/Library/Application Support/Google/Chrome/Default/Bookmarks` (macOS)
- Firefox: `~/Library/Application Support/Firefox/Profiles/*/places.sqlite` (macOS)
- Safari: `~/Library/Safari/Bookmarks.plist` (macOS)

**Import process:**

1. Detect installed browsers
2. Parse bookmark files (JSON for Chrome, SQLite for Firefox, plist for Safari)
3. Extract title, URL, folder hierarchy
4. Store in local database with source attribution
5. Deduplicate by URL

**Why:**

- Direct file parsing is fast and reliable
- No browser extensions needed
- Works offline
- One-time import + manual updates

**Alternatives considered:**

- Browser extensions: Rejected due to installation friction
- Bookmark sync services: Rejected to keep v1 simple
- Continuous sync: Deferred to v2

### 8. UI Component Architecture

**Decision:** HeadlessUI Combobox + custom styling

**Component hierarchy:**

```
LauncherWindow
├── SearchInput (Combobox.Input)
├── ResultsList (Combobox.Options)
│   ├── ResultItem (Combobox.Option)
│   │   ├── Icon
│   │   ├── Title
│   │   ├── Subtitle (URL/Path)
│   │   └── Badge (Type indicator)
├── EmptyState
└── LoadingState
```

**Styling approach:**

- Tailwind CSS for utility-first styling
- Custom color palette inspired by Raycast:
  - Background: `#1a1a1a` (dark) / `#ffffff` (light)
  - Input: `#2a2a2a` (dark) / `#f5f5f5` (light)
  - Accent: `#ff6b6b` (primary action color)
  - Text: `#e0e0e0` (dark) / `#1a1a1a` (light)

**Why:**

- HeadlessUI handles accessibility and keyboard navigation
- Custom styling provides unique brand identity
- Component composition enables easy extension

### 9. Frontend-Backend Communication

**Decision:** Tauri commands (Rust backend ↔ Vue frontend)

**API surface:**

```rust
// Search
#[tauri::command]
async fn search_resources(query: String) -> Result<Vec<SearchResult>, String>

// Bookmarks
#[tauri::command]
async fn import_bookmarks(browser: String) -> Result<usize, String>
#[tauri::command]
async fn add_bookmark(title: String, url: String) -> Result<(), String>
#[tauri::command]
async fn delete_bookmark(id: i64) -> Result<(), String>

// Files
#[tauri::command]
async fn set_search_directories(paths: Vec<String>) -> Result<(), String>
#[tauri::command]
async fn reindex_files() -> Result<(), String>

// Resources
#[tauri::command]
async fn open_resource(resource_type: String, id: i64) -> Result<(), String>

// Settings
#[tauri::command]
async fn get_settings() -> Result<HashMap<String, String>, String>
#[tauri::command]
async fn update_setting(key: String, value: String) -> Result<(), String>
```

**Why:**

- Type-safe communication via serde
- Async by default for non-blocking operations
- Error handling built-in
- Minimal boilerplate

## Risks / Trade-offs

### Risk: Search performance degradation with large datasets

**Mitigation:**

- Limit initial file indexing to 100K files
- Use database indexes and FTS5 optimization
- Implement pagination (show top 10 results)
- Profile and optimize hot paths
- Add performance monitoring

### Risk: Cross-platform inconsistencies

**Mitigation:**

- Early testing on all target platforms
- Use Tauri's platform abstraction
- Platform-specific code only where necessary
- Document platform differences

### Risk: Browser bookmark format changes

**Mitigation:**

- Version detection for bookmark files
- Graceful degradation on parse errors
- Manual bookmark entry as fallback
- Clear error messages to users

### Risk: File system watcher reliability

**Mitigation:**

- Periodic background re-scans (configurable, default: daily)
- Manual re-index option
- Detect watcher failures and alert user
- Graceful degradation to polling if watchers fail

## Migration Plan

N/A - This is a new application with no existing users or data to migrate.

**Future migrations (v2+):**

- Settings schema changes: Use migrations in SQLite
- Data model changes: Provide migration scripts
- Breaking changes: Maintain backwards compatibility for 1 major version

## Open Questions

1. **Should we support custom search plugins?**
   - Decision: No for v1, revisit in v2 based on user feedback

2. **Should we index file contents (not just names)?**
   - Decision: No for v1 (performance concerns), consider for v2

3. **How to handle bookmark conflicts during import?**
   - Decision: Keep existing, skip duplicates, log conflicts

4. **Should we support cloud sync?**
   - Decision: No for v1, consider third-party sync (Dropbox, iCloud) in v2

5. **What about Windows/Linux-specific features (e.g., Windows Search integration)?**
   - Decision: Keep parity across platforms for v1, platform-specific optimizations in v2
