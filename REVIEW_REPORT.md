# Code Review & Optimization Report

## Overview
This report outlines findings from a code review of the `launcher-app`, focusing on performance, architecture, and code quality.

## 1. Performance Optimizations

### 1.1 Backend: Database Concurrency (Critical)
**Issue:** The SQLite database uses the default journaling mode (`DELETE`), which locks the entire database file during write operations. This prevents concurrent reads, potentially blocking the UI during background indexing.
**Location:** `src-tauri/src/db/mod.rs`
**Recommendation:** Enable Write-Ahead Logging (WAL) mode.
**Proposed Change:**
In `Database::initialize`:
```rust
self.conn.execute_batch(
    r#"
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = NORMAL;
    -- ... (rest of the schema)
    "#
)?;
```

### 1.2 Frontend: Search Debounce (High)
**Issue:** The search input debounce is set to **50ms**.
**Location:** `src/components/SearchCombobox.vue`
**Impact:** This results in excessive IPC calls and search queries while typing, wasting resources and potentially causing UI jank.
**Recommendation:** Increase debounce to **150ms** or **200ms**.

## 2. Architecture Observations

### 2.1 Connection Management
The application creates multiple `rusqlite::Connection` instances (one in `DataService`, one in `AppState`, one for macOS dock checks). Without WAL mode, this is a significant contention risk. With WAL mode, it is acceptable but a Connection Pool (or shared Singleton) would be more robust for the long term.

### 2.2 Search Engine
The `TantivySearchEngine` correctly implements a "Zero-Lock" architecture, where search operations read directly from the memory-mapped index without acquiring database locks. This is a strong design choice that should be preserved.

## 3. Code Quality

### 3.1 Static Analysis
- **Frontend:** `vue-tsc` passed with no errors.
- **Backend:** Code structure is clean and modular.

## 4. Next Steps
1. Apply the WAL mode patch.
2. Adjust the frontend debounce timer.
