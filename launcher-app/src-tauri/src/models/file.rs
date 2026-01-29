use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFile {
    pub id: Option<i64>,
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size: i64,
    pub modified_at: i64,
    pub created_at: i64,
    pub indexed_at: i64,
    pub directory_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSearchResult {
    pub id: i64,
    pub path: String,
    pub name: String,
    pub extension: Option<String>,
    pub size: i64,
    pub modified_at: i64,
    pub score: f64,
    pub frecency_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDirectory {
    pub id: Option<i64>,
    pub path: String,
    pub enabled: bool,
    pub include_hidden: bool,
    pub created_at: i64,
    pub last_indexed_at: Option<i64>,
    pub file_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingProgress {
    pub directory_id: i64,
    pub directory_path: String,
    pub files_scanned: usize,
    pub files_indexed: usize,
    pub is_complete: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryValidationResult {
    pub valid: bool,
    pub error: Option<String>,
    pub estimated_file_count: Option<usize>,
    pub requires_confirmation: bool,
}
