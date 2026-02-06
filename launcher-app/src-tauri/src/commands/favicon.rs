use crate::commands::bookmarks::AppState;
use tauri::State;
use std::time::Duration;
use url;

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

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    match client.get(&favicon_url).send().await {
        Ok(response) if response.status().is_success() => {
            let db = state.db.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
            let conn = db.get_connection();
            
            conn.execute(
                "UPDATE bookmarks SET favicon_url = ?1 WHERE id = ?2",
                rusqlite::params![&favicon_url, bookmark_id],
            )
            .map_err(|e| format!("Failed to update favicon: {}", e))?;
            
            Ok(favicon_url)
        }
        _ => {
            let google_favicon = format!("https://www.google.com/s2/favicons?domain={}&sz=64", base_url);
            
            let db = state.db.lock().map_err(|e| format!("Failed to acquire lock: {}", e))?;
            let conn = db.get_connection();
            
            conn.execute(
                "UPDATE bookmarks SET favicon_url = ?1 WHERE id = ?2",
                rusqlite::params![&google_favicon, bookmark_id],
            )
            .map_err(|e| format!("Failed to update favicon: {}", e))?;
            
            Ok(google_favicon)
        }
    }
}
