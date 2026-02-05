//! Application-wide error types
//!
//! This module defines a unified error type for the entire application,
//! replacing scattered String errors with proper, type-safe error handling.

use thiserror::Error;

/// Application-level error type
///
/// This enum covers all error cases across the application, providing
/// better error handling than String-based errors.
#[derive(Error, Debug)]
pub enum AppError {
    // Database errors
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Failed to acquire database lock")]
    DatabaseLock,

    #[error("Bookmark not found")]
    BookmarkNotFound,

    #[error("Bookmark with URL '{0}' already exists")]
    DuplicateBookmark(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    // Search engine errors
    #[error("Search error: {0}")]
    Search(String),

    #[error("Failed to index bookmark {0}: {1}")]
    IndexingFailed(i64, String),

    #[error("Failed to rebuild index: {0}")]
    RebuildFailed(String),

    // I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to read directory: {0}")]
    DirectoryRead(String),

    // System errors
    #[error("System clock error")]
    SystemTime(#[from] std::time::SystemTimeError),

    #[error("Failed to get app data directory: {0}")]
    AppDataDir(String),

    // Import errors
    #[error("Chrome import failed: {0}")]
    ChromeImport(String),

    #[error("Firefox import failed: {0}")]
    FirefoxImport(String),

    #[error("Safari import failed: {0}")]
    SafariImport(String),

    // URL and network errors
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    // JSON serialization errors
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    // Settings errors
    #[error("Setting '{0}' not found")]
    SettingNotFound(String),

    #[error("Invalid setting value for '{0}': {1}")]
    InvalidSettingValue(String, String),

    // Generic errors (use sparingly)
    #[error("{0}")]
    Generic(String),
}

/// Result type alias for application code
pub type AppResult<T> = Result<T, AppError>;

impl From<String> for AppError {
    fn from(s: String) -> Self {
        AppError::Generic(s)
    }
}

impl From<&str> for AppError {
    fn from(s: &str) -> Self {
        AppError::Generic(s.to_string())
    }
}

// Implement conversion to String for Tauri commands
// Tauri commands require Result<T, String>, so we provide this conversion
impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = AppError::BookmarkNotFound;
        assert_eq!(err.to_string(), "Bookmark not found");

        let err = AppError::DuplicateBookmark("https://example.com".to_string());
        assert_eq!(
            err.to_string(),
            "Bookmark with URL 'https://example.com' already exists"
        );
    }

    #[test]
    fn test_error_conversion() {
        let err: AppError = "test error".into();
        assert_eq!(err.to_string(), "test error");

        let err: AppError = "another error".to_string().into();
        assert_eq!(err.to_string(), "another error");
    }

    #[test]
    fn test_error_to_string_conversion() {
        let err = AppError::BookmarkNotFound;
        let s: String = err.into();
        assert_eq!(s, "Bookmark not found");
    }
}
