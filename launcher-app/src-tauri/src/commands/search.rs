use crate::commands::bookmarks::{validate_limit, AppState};
use crate::models::bookmark::BookmarkSearchResult;
use crate::search::provider::SourceType;
use crate::search::{IndexStats, SearchContext};
use crate::search::query_parser::StructuredQuery;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Clone, Serialize)]
pub struct UnifiedSearchResult {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub source_type: SourceType,
    pub source_id: String,
    pub score: f64,
    pub frecency_score: f64,
    pub icon: Option<String>,
    pub url: Option<String>,
    pub path: Option<String>,
    pub favicon_url: Option<String>,
    pub description: Option<String>,
    pub extension: Option<String>,
    pub size: Option<i64>,
    pub modified_at: Option<i64>,
    pub plugin_actions: Option<Vec<serde_json::Value>>,
    pub plugin_badge: Option<String>,
    pub plugin_keyword: Option<String>,
}

#[tauri::command]
pub async fn unified_search(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
    sources: Option<Vec<String>>,
) -> Result<Vec<UnifiedSearchResult>, String> {
    let effective_limit = validate_limit(limit, 100)?;

    // Read search settings from settings store (no more DB query)
    let search_settings = {
        let cache = state.settings_cache.read().map_err(|e| e.to_string())?;

        let cache_has_values = cache.contains_key("search.max_results")
            || cache.contains_key("search.show_bookmarks")
            || cache.contains_key("search.show_files")
            || cache.contains_key("search.fuzzy_matching");

        if !cache_has_values {
            drop(cache);
            // Populate cache from settings store
            let _ = state.data_service.with_settings_store(|store| {
                let flat_map = store.to_flat_map();
                if let Ok(mut write_cache) = state.settings_cache.write() {
                    for key in &[
                        "search.max_results",
                        "search.show_bookmarks",
                        "search.show_files",
                        "search.fuzzy_matching",
                    ] {
                        if let Some(v) = flat_map.get(*key) {
                            write_cache.insert(key.to_string(), v.clone());
                        }
                    }
                }
                Ok(())
            });
        }

        let cache = state.settings_cache.read().map_err(|e| e.to_string())?;

        let max_results: usize = cache
            .get("search.max_results")
            .and_then(|s| s.parse().ok())
            .unwrap_or(10);
        let show_bookmarks: bool = cache
            .get("search.show_bookmarks")
            .map(|s| s == "true")
            .unwrap_or(true);
        let show_files: bool = cache
            .get("search.show_files")
            .map(|s| s == "true")
            .unwrap_or(true);
        let fuzzy_matching: bool = cache
            .get("search.fuzzy_matching")
            .map(|s| s == "true")
            .unwrap_or(true);

        (max_results, show_bookmarks, show_files, fuzzy_matching)
    };

    let (_, show_bookmarks, show_files, fuzzy_matching) = search_settings;

    let effective_sources = if let Some(explicit) = sources {
        Some(explicit)
    } else {
        let mut enabled = Vec::new();
        if show_bookmarks { enabled.push("bookmarks".to_string()); }
        if show_files { enabled.push("files".to_string()); }
        enabled.push("plugins".to_string());
        Some(enabled)
    };

    let structured_query = StructuredQuery::parse(&query);

    let ctx = SearchContext {
        query,
        structured_query,
        limit: effective_limit,
        fuzzy: fuzzy_matching,
        sources: effective_sources,
    };

    let results = state.search_aggregator.search(&ctx).await
        .map_err(|e| format!("Unified search failed: {}", e))?;

    Ok(results
        .into_iter()
        .map(|r| UnifiedSearchResult {
            id: r.id,
            title: r.title,
            subtitle: r.subtitle,
            source_type: r.source_type,
            source_id: r.source_id,
            score: r.score,
            frecency_score: r.frecency_score,
            icon: r.icon,
            url: r.url,
            path: r.path,
            favicon_url: r.favicon_url,
            description: r.description,
            extension: r.extension,
            size: r.size,
            modified_at: r.modified_at,
            plugin_actions: r.plugin_actions,
            plugin_badge: r.plugin_badge,
            plugin_keyword: r.plugin_keyword,
        })
        .collect())
}

#[tauri::command]
pub fn search_bookmarks(
    state: State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Result<Vec<BookmarkSearchResult>, String> {
    let limit = limit.unwrap_or(10);

    let results = state
        .data_service
        .search_engine()
        .search_bookmarks(&query, limit)
        .map_err(|e| format!("Search failed: {}", e))?;

    Ok(results)
}

#[tauri::command]
pub fn record_bookmark_access(state: State<AppState>, bookmark_id: i64) -> Result<(), String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("System clock error: {}", e))?
        .as_secs() as i64;

    // Update bookmark in store
    state
        .data_service
        .with_bookmark_store_mut(|store| {
            store
                .update_last_accessed(bookmark_id, now)
                .map_err(|e| crate::error::AppError::Generic(e))
        })
        .map_err(|e| e.to_string())?;

    // Update Tantivy index with new frecency data (fire and forget)
    // We don't track individual access events anymore — just increment count
    let _ = state.data_service.search_engine().update_bookmark_frecency(bookmark_id, 1, now);

    Ok(())
}

#[tauri::command]
pub fn rebuild_search_index(state: State<AppState>) -> Result<(usize, usize), String> {
    let (bookmarks, _files) = state
        .data_service
        .get_all_index_data()
        .map_err(|e| format!("Failed to fetch index data: {}", e))?;

    let bookmark_count = state
        .data_service
        .search_engine()
        .rebuild_bookmark_index_from_data(bookmarks)
        .map_err(|e| format!("Failed to rebuild bookmark index: {}", e))?;

    // File index rebuild requires re-scanning directories
    // Return 0 for file count — caller should trigger directory re-indexing separately
    Ok((bookmark_count, 0))
}

/// Refresh the settings cache from settings store.
#[tauri::command]
pub fn refresh_settings_cache(state: State<AppState>) -> Result<(), String> {
    let flat_map = state
        .data_service
        .with_settings_store(|store| Ok(store.to_flat_map()))
        .map_err(|e| e.to_string())?;

    if let Ok(mut cache) = state.settings_cache.write() {
        cache.clear();
        for key in &[
            "search.max_results",
            "search.show_bookmarks",
            "search.show_files",
            "search.fuzzy_matching",
        ] {
            if let Some(v) = flat_map.get(*key) {
                cache.insert(key.to_string(), v.clone());
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_search_stats(state: State<AppState>) -> Result<IndexStats, String> {
    state
        .data_service
        .search_engine()
        .get_stats()
        .map_err(|e| format!("Failed to get search stats: {}", e))
}
