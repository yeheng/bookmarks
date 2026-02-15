use crate::commands::bookmarks::AppState;
use tauri::State;

#[tauri::command]
pub async fn fetch_favicon(
    state: State<'_, AppState>,
    bookmark_id: i64,
    url_str: String,
) -> Result<String, String> {
    let parsed_url = url::Url::parse(&url_str)
        .map_err(|e| format!("Invalid URL: {}", e))?;

    let base_url = format!(
        "{}://{}",
        parsed_url.scheme(),
        parsed_url.host_str().unwrap_or("")
    );

    let favicon_url = format!("{}/favicon.ico", base_url);

    let final_url = match state.http_client.get(&favicon_url).send().await {
        Ok(response) if response.status().is_success() => favicon_url,
        _ => format!("https://www.google.com/s2/favicons?domain={}&sz=64", base_url),
    };

    // Update the bookmark's favicon_url in the store
    state
        .data_service
        .with_bookmark_store_mut(|store| {
            if let Some(bookmark) = store.bookmarks_mut().iter_mut().find(|b| b.id == Some(bookmark_id)) {
                bookmark.favicon_url = Some(final_url.clone());
            }
            store.save().map_err(|e| crate::error::AppError::Generic(e))
        })
        .map_err(|e| e.to_string())?;

    Ok(final_url)
}
