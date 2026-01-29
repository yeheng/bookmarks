use tauri::{AppHandle, Manager, WebviewWindow};

#[tauri::command]
pub fn toggle_window(app: AppHandle) -> Result<(), String> {
    let window = app.get_webview_window("main").ok_or("Window not found")?;

    if window.is_visible().unwrap_or(false) {
        hide_window(window)?;
    } else {
        show_window(window)?;
    }

    Ok(())
}

#[tauri::command]
pub fn show_window(window: WebviewWindow) -> Result<(), String> {
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn hide_window(window: WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())?;
    Ok(())
}
