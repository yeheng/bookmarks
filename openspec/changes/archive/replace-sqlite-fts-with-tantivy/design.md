# Design Document: Tantivy Search Engine Integration

## Context

The current implementation uses SQLite FTS5 for full-text search. While functional, FTS5 has limitations in tokenization (especially for CJK languages), fuzzy matching, and search customization. Tantivy is a Rust-native search engine that offers better control over these aspects while maintaining excellent performance.

**Constraints:**

- Performance: Must maintain <100ms search latency
- Compatibility: Keep existing frontend API unchanged
- Data integrity: SQLite remains source of truth
- Resource efficiency: Index size should be reasonable (<10% of data size)

**Stakeholders:**

- End users: Benefit from better search quality
- Developers: New search module to maintain

## Goals / Non-Goals

**Goals:**

- Replace FTS5 with Tantivy for better search capabilities
- Create clean abstraction layer for search operations
- Maintain backward compatibility with existing API
- Support fuzzy matching and prefix search
- Enable future CJK tokenization support

**Non-Goals:**

- Content indexing (searching inside files) - deferred
- Distributed search - not needed for desktop app
- Real-time streaming search - not needed
- Custom scoring plugins - keep simple for v1

## Decisions

### 1. Architecture Pattern

**Decision:** Dual-storage architecture with SQLite as data store and Tantivy as search index

```
┌─────────────────────────────────────────────────────────┐
│                    COMMAND LAYER                         │
│     search.rs          │         file_search.rs         │
└────────────────────────┬────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────┐
│              SearchEngine Trait (Abstraction)            │
│  - search_bookmarks()    - index_bookmark()             │
│  - search_files()        - index_file()                 │
│  - delete_*()            - rebuild_*_index()            │
└────────────────────────┬────────────────────────────────┘
                         │
          ┌──────────────┴──────────────┐
          ▼                              ▼
┌──────────────────┐          ┌──────────────────┐
│ SQLite Database  │          │  Tantivy Index   │
│ (Data Storage)   │◄────────►│ (Search Index)   │
│ - bookmarks      │  sync    │ - bookmarks/     │
│ - indexed_files  │          │ - files/         │
└──────────────────┘          └──────────────────┘
```

**Why:**

- Clear separation of concerns
- SQLite handles ACID transactions for data
- Tantivy optimized for search operations
- Easy to rebuild index from source of truth

**Alternatives considered:**

- Tantivy-only storage: Rejected due to loss of SQL query flexibility
- Keep FTS5 as fallback: Rejected to avoid complexity

### 2. Tantivy Schema Design

**Decision:** Separate schemas for bookmarks and files with appropriate tokenizers

**Bookmark Schema:**

```rust
// Stored fields (returned in results)
id: i64 (STORED | FAST)
title: TEXT (STORED)
url: TEXT (STORED)
description: TEXT (STORED)

// Indexed fields (searchable)
title_indexed: TEXT (tokenized with en_stem)
url_indexed: TEXT (tokenized with en_stem)
description_indexed: TEXT (tokenized with en_stem)
tags: TEXT (tokenized)

// Numeric fields for filtering/sorting
last_accessed: i64 (FAST)
created_at: i64 (FAST)
updated_at: i64 (FAST)
```

**File Schema:**

```rust
// Stored fields
id: i64 (STORED | FAST)
path: TEXT (STORED)
name: TEXT (STORED)
extension: STRING (STORED)
size: i64 (STORED | FAST)
modified_at: i64 (STORED | FAST)

// Indexed fields with ngram for fuzzy matching
name_indexed: TEXT (ngram3 tokenizer)
path_indexed: TEXT (ngram3 tokenizer)

// Fast field for extension filtering
extension_facet: STRING (FAST)
directory_id: i64 (FAST)
```

**Why:**

- Ngram tokenizer (3-gram) enables fuzzy file name matching
- Stemming for bookmark text improves recall
- FAST fields enable efficient filtering and sorting
- Separate stored/indexed fields for flexibility

### 3. Index Update Strategy

**Decision:** Immediate updates with async commit

**Strategy:**

1. CRUD operation updates SQLite first (source of truth)
2. Then update Tantivy index
3. Commit changes immediately for visibility
4. Use delete-then-add pattern for updates

```rust
async fn index_bookmark(&self, id: i64, ...) -> Result<()> {
    let mut writer = self.writer.lock().unwrap();

    // Delete existing document if present
    let term = Term::from_field_i64(id_field, id);
    writer.delete_term(term);

    // Add new document
    let mut doc = Document::new();
    // ... add fields
    writer.add_document(doc)?;

    // Commit immediately
    writer.commit()?;
    Ok(())
}
```

**Why:**

- Immediate consistency between data and search
- Simple mental model for developers
- Acceptable performance for desktop app scale

**Alternatives considered:**

- Batched commits: Better performance but added complexity
- Background sync: Risk of stale search results

### 4. Frecency Integration

**Decision:** Hybrid scoring with Tantivy BM25 + SQLite frecency

**Formula:**

```
final_score = tantivy_bm25_score * 0.7 + frecency_score * 0.3
```

**Implementation:**

1. Tantivy returns top N*2 results with BM25 scores
2. Query SQLite for frecency scores of matched IDs
3. Combine scores in Rust
4. Sort and return top N

**Why:**

- Tantivy handles text relevance
- SQLite maintains access history (already there)
- Combining in Rust avoids complex index updates for every access

### 5. Index Initialization

**Decision:** Background index build on startup with progress reporting

**Flow:**

```
App Start
    │
    ▼
Check if index exists?
    │
    ├─ Yes → Verify index health → Ready
    │
    └─ No → Start background rebuild
             │
             ▼
         Report progress to frontend
             │
             ▼
         Index ready event
```

**Why:**

- Non-blocking startup
- User can search already-indexed items
- Clear progress feedback

### 6. Error Handling

**Decision:** Graceful degradation with empty results

**Strategy:**

1. If Tantivy search fails, log error and return empty results
2. If indexing fails, retry once, then skip item
3. Provide manual rebuild command for recovery
4. Never crash the app due to search issues

**Why:**

- Search is not critical path for data integrity
- Better UX than error dialogs
- Manual recovery option available

## Module Structure

```
src-tauri/src/
├── search/
│   ├── mod.rs              # Module exports
│   ├── engine.rs           # SearchEngine trait definition
│   ├── schema.rs           # Tantivy schema builders
│   ├── tantivy_engine.rs   # TantivySearchEngine implementation
│   └── tests.rs            # Integration tests
```

## Risks / Trade-offs

### Risk: Index corruption

**Mitigation:**

- SQLite remains source of truth
- Rebuild command available
- Index health check on startup

### Risk: Increased disk usage

**Mitigation:**

- Tantivy indexes are compact (~10-20% of source data)
- Single index per category (not per-field)
- Periodic index optimization

### Risk: Cold start performance

**Mitigation:**

- Use mmap for index access (fast loading)
- Background initialization
- Index persistence between runs

### Risk: Memory usage

**Mitigation:**

- Configure writer heap size (50MB default)
- Reader uses mmap (OS manages memory)
- Single writer per index

## Migration Plan

### Phase 1: Setup (No Breaking Changes)

1. Add tantivy dependency to Cargo.toml
2. Create search module with all new code
3. Add unit tests for new components

### Phase 2: Parallel Running

1. Initialize Tantivy alongside FTS5
2. Build initial index from SQLite data
3. Run both systems, compare results (dev only)

### Phase 3: Cutover

1. Switch search commands to use Tantivy
2. Update CRUD operations to maintain Tantivy index
3. Keep FTS5 tables temporarily

### Phase 4: Cleanup

1. Remove FTS5 virtual tables and triggers
2. Remove FTS5-related code
3. Update documentation

## Open Questions

1. **Should we support custom tokenizers for CJK?**
   - Decision: Prepare the architecture but implement in follow-up change
   - Tantivy supports jieba tokenizer for Chinese

2. **Index location configurability?**
   - Decision: Keep in app data directory for v1
   - Could add setting later if needed

3. **Index versioning for schema changes?**
   - Decision: Full rebuild on schema change for v1
   - Consider migration support in v2
