//! Application-wide error types

use thiserror::Error;

/// Application-level error type
#[derive(Error, Debug)]
pub enum AppError {
    // Store errors
    #[error("Failed to acquire store lock")]
    StoreLock,

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

/// Error codes for programmatic error handling in frontend
impl AppError {
    /// Get a numeric error code for this error type
    pub fn code(&self) -> u32 {
        match self {
            AppError::StoreLock => 1002,
            AppError::BookmarkNotFound => 2001,
            AppError::DuplicateBookmark(_) => 2002,
            AppError::FileNotFound(_) => 3001,
            AppError::Search(_) => 4001,
            AppError::IndexingFailed(_, _) => 4002,
            AppError::RebuildFailed(_) => 4003,
            AppError::Io(_) => 5001,
            AppError::DirectoryRead(_) => 5002,
            AppError::SystemTime(_) => 6001,
            AppError::AppDataDir(_) => 6002,
            AppError::ChromeImport(_) => 7001,
            AppError::FirefoxImport(_) => 7002,
            AppError::SafariImport(_) => 7003,
            AppError::InvalidUrl(_) => 8001,
            AppError::Network(_) => 8002,
            AppError::Json(_) => 9001,
            AppError::SettingNotFound(_) => 10001,
            AppError::InvalidSettingValue(_, _) => 10002,
            AppError::Generic(_) => 11001,
        }
    }
}

/// Structured error response for API responses
#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    pub code: u32,
    pub message: String,
}

impl From<AppError> for ErrorResponse {
    fn from(err: AppError) -> Self {
        ErrorResponse {
            code: err.code(),
            message: err.to_string(),
        }
    }
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
