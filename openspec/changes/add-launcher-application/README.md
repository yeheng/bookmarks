# OpenSpec Change Proposal: Desktop Launcher Application

## Summary

This proposal defines a new desktop productivity launcher application similar to Raycast, Alfred, and Spotlight. The application provides instant access to bookmarks and files through a global hotkey with real-time search (<100ms response time).

## Proposal Status

- **Change ID**: `add-launcher-application`
- **Status**: ✅ Validated (strict mode)
- **Capabilities**: 4 new capabilities
- **Requirements**: 21 total requirements
- **Validation**: Passed `openspec validate --strict`

## Capabilities

### 1. global-launcher

Core application framework including:

- Global hotkey registration (Cmd/Ctrl+Space)
- Window visibility management with auto-hide
- Frameless, transparent window design
- Cross-platform support (macOS, Windows, Linux)

**Requirements**: 4

### 2. bookmark-search

Bookmark management and search:

- Browser import (Chrome, Firefox, Safari)
- Manual bookmark CRUD operations
- Real-time search with frecency ranking
- Favicon display and storage
- SQLite database with FTS5

**Requirements**: 6

### 3. file-search

File indexing and search:

- User-configurable search directories
- Background file system indexing
- Incremental updates via file watchers
- Fuzzy search with metadata display
- SQLite persistence with FTS5

**Requirements**: 6

### 4. ui-framework

Minimalist search interface:

- HeadlessUI/Vue components (Combobox)
- Real-time search feedback (<100ms)
- Keyboard navigation and accessibility
- Dark/light theme support
- Smooth animations and transitions

**Requirements**: 5

## Technical Highlights

### Technology Stack

- **Frontend**: Vue.js 3 + TypeScript + HeadlessUI + Tailwind CSS
- **Backend**: Rust (Tauri 2.x)
- **Database**: SQLite with FTS5 full-text search
- **Cross-platform**: macOS 10.15+, Windows 10+, Linux (X11/Wayland)

### Performance Targets

- Search response time: <100ms
- Idle memory usage: <100MB
- Window transitions: 150ms fade
- File index limit: 100,000 files per directory

### Search Algorithm

Hybrid approach:

```
final_score = (fts5_score * 0.7) + (frecency_score * 0.3)
frecency_score = (frequency * 0.6) + (recency * 0.4)
```

## Files Structure

```
openspec/changes/add-launcher-application/
├── proposal.md                      # Why, what, impact
├── tasks.md                         # Implementation checklist (9 sections, 57 tasks)
├── design.md                        # Technical decisions and architecture
└── specs/
    ├── global-launcher/spec.md      # 4 requirements, 13 scenarios
    ├── bookmark-search/spec.md      # 6 requirements, 21 scenarios
    ├── file-search/spec.md          # 6 requirements, 19 scenarios
    └── ui-framework/spec.md         # 5 requirements, 18 scenarios
```

## Implementation Phases

### Phase 1: Project Setup (Tasks 1.1-1.5)

- Initialize Tauri + Vue.js 3 project
- Configure TypeScript, HeadlessUI, Tailwind CSS
- Set up project structure

### Phase 2: Core Launcher (Tasks 2.1-2.5)

- Global hotkey registration
- Window management and visibility
- Frameless window with transparency

### Phase 3: UI Framework (Tasks 3.1-3.6)

- Search input component
- Results list with keyboard navigation
- Minimalist theme and animations

### Phase 4: Bookmark Search (Tasks 4.1-4.8)

- Database schema and storage
- Browser import (Chrome, Firefox, Safari)
- Manual bookmark management
- Full-text search with ranking

### Phase 5: File Search (Tasks 5.1-5.7)

- Directory configuration
- File system indexing
- Incremental updates with watchers
- Fuzzy search algorithm

### Phase 6: Resource Opening (Tasks 6.1-6.4)

- URL opening in browser
- File opening in default apps
- Error handling and history

### Phase 7: Settings (Tasks 7.1-7.5)

- Settings panel UI
- Hotkey customization
- Theme and path management

### Phase 8: Testing (Tasks 8.1-8.6)

- Unit and integration tests
- Performance testing
- Cross-platform testing

### Phase 9: Distribution (Tasks 9.1-9.5)

- Documentation
- Application signing
- Auto-update mechanism

## Key Decisions (from design.md)

1. **Tauri over Electron**: 3-5MB bundle vs 100MB+, better performance
2. **SQLite FTS5**: Fast full-text search with <1M records, local-first
3. **Hybrid search**: FTS5 + frecency for relevance + personalization
4. **Incremental indexing**: File watchers + periodic re-scans
5. **Direct browser parsing**: No extensions needed, fast and reliable
6. **HeadlessUI components**: Accessible, keyboard-first, customizable

## Next Steps

1. ✅ Proposal created and validated
2. ⏳ **Awaiting approval** - Review proposal, design, and specs
3. ⏳ Implementation - Follow tasks.md sequentially
4. ⏳ Testing and quality assurance
5. ⏳ Distribution and deployment

## Validation

```bash
$ openspec validate add-launcher-application --strict
Change 'add-launcher-application' is valid
```

## Questions or Feedback?

Please review:

- `proposal.md` - High-level overview and impact
- `design.md` - Technical decisions and trade-offs
- `tasks.md` - Implementation plan (57 tasks)
- `specs/*/spec.md` - Detailed requirements and scenarios (21 requirements, 71 scenarios)

---

**Created**: 2026-01-29  
**Status**: Ready for approval  
**Author**: OpenCode AI Assistant
