use crate::commands::bookmarks::AppState;
use crate::error::AppError;
use tauri::State;
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

    match state.http_client.get(&favicon_url).send().await {
        Ok(response) if response.status().is_success() => {
            state.data_service.with_db(|conn| {
                conn.execute(
                    "UPDATE bookmarks SET favicon_url = ?1 WHERE id = ?2",
                    rusqlite::params![&favicon_url, bookmark_id],
                )
                .map_err(|e| AppError::Generic(format!("Failed to update favicon: {}", e)))?;

                Ok(())
            }).map_err(|e| e.to_string())?;

            Ok(favicon_url)
        }
        _ => {
            let google_favicon = format!("https://www.google.com/s2/favicons?domain={}&sz=64", base_url);

            state.data_service.with_db(|conn| {
                conn.execute(
                    "UPDATE bookmarks SET favicon_url = ?1 WHERE id = ?2",
                    rusqlite::params![&google_favicon, bookmark_id],
                )
                .map_err(|e| AppError::Generic(format!("Failed to update favicon: {}", e)))?;

                Ok(())
            }).map_err(|e| e.to_string())?;

            Ok(google_favicon)
        }
    }
}
