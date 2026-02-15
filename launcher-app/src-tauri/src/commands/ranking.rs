use crate::commands::bookmarks::AppState;
use tauri::State;

#[tauri::command]
pub fn record_usage(
    state: State<'_, AppState>,
    source_id: String,
    item_id: String,
) -> Result<(), String> {
    state.search_aggregator
        .record_usage(&source_id, &item_id)
        .map_err(|e| e.to_string())
}
