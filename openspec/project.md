# Project Context

## Purpose

Desktop productivity launcher application that provides instant access to bookmarks and files through a global hotkey. Inspired by Raycast, Alfred, and Spotlight, this tool enables users to quickly search and open resources without leaving their workflow.

## Tech Stack

- **Frontend**: Vue.js 3, TypeScript, HeadlessUI/Vue, Tailwind CSS
- **Backend**: Rust (via Tauri 2.x)
- **Database**: JSON files (structured data) + Tantivy (full-text search & indexing)
- **Desktop Framework**: Tauri 2.x
- **Build Tools**: Vite, pnpm
- **Platforms**: macOS, Windows, Linux

## Project Conventions

### Code Style

**TypeScript/Vue:**

- Use Composition API with `<script setup>` syntax
- Strict TypeScript mode enabled
- ESLint + Prettier for code formatting
- Vue 3 style guide (official recommendations)
- Naming: PascalCase for components, camelCase for variables/functions

**Rust:**

- Follow Rust standard style (rustfmt)
- Use async/await for I/O operations
- Tauri commands should be well-typed with serde
- Error handling: Result types, avoid unwrap() in production code

### Architecture Patterns

- **Frontend-Backend Communication**: Tauri commands (type-safe IPC)
- **State Management**: Vue 3 reactivity (ref, reactive, computed)
- **Data Access**: JSON file stores with atomic writes (tmp+rename) in Rust; Tantivy for search indexing
- **Search**: Hybrid Tantivy BM25 + frecency scoring algorithm
- **File Organization**:
  - `/src-tauri/` - Rust backend code
  - `/src/` - Vue.js frontend code
  - `/src/components/` - Reusable Vue components
  - `/src/stores/` - State management
  - `/src/types/` - TypeScript type definitions

### Testing Strategy

- **Unit Tests**: Vitest for TypeScript/Vue, cargo test for Rust
- **Integration Tests**: Test Tauri commands with mock database
- **E2E Tests**: Playwright for critical user flows (search, open resource)
- **Coverage Target**: Minimum 70% for core search logic
- **Performance Tests**: Ensure search responds in < 100ms

### Git Workflow

- **Main Branch**: `main` (production-ready code)
- **Feature Branches**: `feature/{description}` (e.g., `feature/bookmark-import`)
- **Commit Convention**: Conventional Commits
  - `feat:` new features
  - `fix:` bug fixes
  - `refactor:` code refactoring
  - `docs:` documentation
  - `test:` testing
  - `chore:` tooling, dependencies
- **PR Process**: Require review before merge, CI must pass

## Domain Context

### Search Ranking (Frecency)

Frecency combines **frequency** and **recency** to rank search results:

- Frequency: How often a resource is accessed
- Recency: How recently a resource was accessed
- Combined score: `(frequency * 0.6) + (recency * 0.4)`
- Final score: `(fts_score * 0.7) + (frecency_score * 0.3)`

### Browser Bookmark Formats

- **Chrome**: JSON file at `~/Library/Application Support/Google/Chrome/Default/Bookmarks`
- **Firefox**: SQLite database at `~/Library/Application Support/Firefox/Profiles/*/places.sqlite`
- **Safari**: plist file at `~/Library/Safari/Bookmarks.plist`

### File System Indexing

- Index metadata only (name, path, size, modified date)
- Skip hidden files and system directories by default
- Use file system watchers for incremental updates
- Limit to 100K files initially for performance

## Important Constraints

- **Performance**: Search must respond in < 100ms (hard requirement)
- **Memory**: Idle memory usage should be < 100MB
- **File Size Limit**: Index files up to 100,000 per directory
- **Platform Support**: Must work on macOS 10.15+, Windows 10+, Linux (X11/Wayland)
- **Offline-First**: All functionality must work without internet connection
- **Privacy**: No telemetry or data sent to external servers

## External Dependencies

### Core Dependencies

- **Tauri**: v2.x - Desktop application framework
- **Vue.js**: v3.x - Frontend framework
- **HeadlessUI**: Accessible UI components
- **Tailwind CSS**: Utility-first styling
- **Tantivy**: Embedded full-text search

### Rust Crates

- `tauri` - Core framework
- `tantivy` - Full-text search engine
- `serde` / `serde_json` - Serialization (JSON file persistence)
- `rusqlite` - SQLite bindings (migration from old DB + Firefox bookmark import only)
- `tokio` - Async runtime
- `notify` - File system watcher
- `global-hotkey` - Global keyboard shortcuts

### npm Packages

- `vue` - Frontend framework
- `@headlessui/vue` - UI components
- `tailwindcss` - Styling
- `vite` - Build tool
- `vitest` - Testing framework
