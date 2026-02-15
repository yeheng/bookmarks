use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Generic JSON file store with atomic writes.
///
/// Loads a `T` from a JSON file on disk, and persists changes via
/// a write-to-tmp + atomic-rename pattern.
pub struct JsonStore {
    path: PathBuf,
}

impl JsonStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Load data from disk. Returns `Default::default()` if file is missing.
    /// If the file exists but is corrupt, renames it to `.corrupt` and returns default.
    pub fn load<T: DeserializeOwned + Default>(&self) -> T {
        if !self.path.exists() {
            return T::default();
        }

        match fs::read_to_string(&self.path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(data) => data,
                Err(e) => {
                    eprintln!(
                        "[JsonStore] Corrupt file {}: {}. Renaming to .corrupt",
                        self.path.display(),
                        e
                    );
                    let corrupt_path = self.path.with_extension("json.corrupt");
                    let _ = fs::rename(&self.path, &corrupt_path);
                    T::default()
                }
            },
            Err(e) => {
                eprintln!(
                    "[JsonStore] Failed to read {}: {}",
                    self.path.display(),
                    e
                );
                T::default()
            }
        }
    }

    /// Persist data to disk atomically (write to .tmp, then rename).
    pub fn save<T: Serialize>(&self, data: &T) -> Result<(), String> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let tmp_path = self.path.with_extension("json.tmp");
        fs::write(&tmp_path, &json)
            .map_err(|e| format!("Failed to write temp file: {}", e))?;

        fs::rename(&tmp_path, &self.path)
            .map_err(|e| format!("Failed to rename temp file: {}", e))?;

        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
    struct TestData {
        items: Vec<String>,
        count: usize,
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let dir = TempDir::new().unwrap();
        let store = JsonStore::new(dir.path().join("missing.json"));
        let data: TestData = store.load();
        assert_eq!(data, TestData::default());
    }

    #[test]
    fn test_save_and_load() {
        let dir = TempDir::new().unwrap();
        let store = JsonStore::new(dir.path().join("test.json"));

        let data = TestData {
            items: vec!["a".to_string(), "b".to_string()],
            count: 2,
        };

        store.save(&data).unwrap();
        let loaded: TestData = store.load();
        assert_eq!(loaded, data);
    }

    #[test]
    fn test_corrupt_file_recovery() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("corrupt.json");

        // Write invalid JSON
        fs::write(&path, "not valid json {{{").unwrap();

        let store = JsonStore::new(path.clone());
        let data: TestData = store.load();

        // Should return default
        assert_eq!(data, TestData::default());

        // Original file should be renamed
        assert!(!path.exists());
        assert!(path.with_extension("json.corrupt").exists());
    }

    #[test]
    fn test_atomic_write_no_partial() {
        let dir = TempDir::new().unwrap();
        let store = JsonStore::new(dir.path().join("atomic.json"));

        let data = TestData {
            items: vec!["hello".to_string()],
            count: 1,
        };

        store.save(&data).unwrap();

        // Tmp file should not remain
        assert!(!dir.path().join("atomic.json.tmp").exists());
        assert!(dir.path().join("atomic.json").exists());
    }
}
