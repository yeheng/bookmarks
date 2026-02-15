use crate::models::file::SearchDirectory;
use crate::store::json_store::JsonStore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryStoreData {
    pub next_id: i64,
    pub directories: Vec<SearchDirectory>,
}

impl Default for DirectoryStoreData {
    fn default() -> Self {
        Self {
            next_id: 1,
            directories: Vec::new(),
        }
    }
}

pub struct DirectoryStore {
    store: JsonStore,
    data: DirectoryStoreData,
}

impl DirectoryStore {
    pub fn new(path: PathBuf) -> Self {
        let store = JsonStore::new(path);
        let data: DirectoryStoreData = store.load();
        Self { store, data }
    }

    pub fn save(&self) -> Result<(), String> {
        self.store.save(&self.data)
    }

    pub fn directories(&self) -> &[SearchDirectory] {
        &self.data.directories
    }

    pub fn count(&self) -> usize {
        self.data.directories.len()
    }

    pub fn enabled_count(&self) -> usize {
        self.data.directories.iter().filter(|d| d.enabled).count()
    }

    pub fn find_by_id(&self, id: i64) -> Option<&SearchDirectory> {
        self.data.directories.iter().find(|d| d.id == Some(id))
    }

    pub fn find_by_path(&self, path: &str) -> Option<&SearchDirectory> {
        self.data.directories.iter().find(|d| d.path == path)
    }

    pub fn add(
        &mut self,
        path: String,
        include_hidden: bool,
    ) -> Result<SearchDirectory, String> {
        if self.find_by_path(&path).is_some() {
            return Err("Directory is already being indexed".to_string());
        }

        let now = now_ts();
        let id = self.data.next_id;
        self.data.next_id += 1;

        let dir = SearchDirectory {
            id: Some(id),
            path,
            enabled: true,
            include_hidden,
            created_at: now,
            last_indexed_at: None,
            file_count: 0,
        };

        self.data.directories.push(dir.clone());
        self.save()?;
        Ok(dir)
    }

    pub fn remove(&mut self, id: i64) -> Result<(), String> {
        let len_before = self.data.directories.len();
        self.data.directories.retain(|d| d.id != Some(id));

        if self.data.directories.len() == len_before {
            return Err("Directory not found".to_string());
        }

        self.save()
    }

    pub fn toggle(&mut self, id: i64, enabled: bool) -> Result<(), String> {
        let dir = self
            .data
            .directories
            .iter_mut()
            .find(|d| d.id == Some(id))
            .ok_or("Directory not found")?;

        dir.enabled = enabled;
        self.save()
    }

    pub fn update_index_stats(
        &mut self,
        id: i64,
        file_count: i64,
        last_indexed_at: i64,
    ) -> Result<(), String> {
        let dir = self
            .data
            .directories
            .iter_mut()
            .find(|d| d.id == Some(id))
            .ok_or("Directory not found")?;

        dir.file_count = file_count;
        dir.last_indexed_at = Some(last_indexed_at);
        self.save()
    }

    /// Get sorted directories (by created_at descending, matching the old SQL ORDER BY).
    pub fn get_sorted(&self) -> Vec<SearchDirectory> {
        let mut dirs = self.data.directories.clone();
        dirs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        dirs
    }

    /// Get enabled directory paths for export.
    pub fn get_enabled_paths(&self) -> Vec<String> {
        self.data
            .directories
            .iter()
            .filter(|d| d.enabled)
            .map(|d| d.path.clone())
            .collect()
    }

    /// Import a directory (for data import). Returns true if added, false if already exists.
    pub fn import_directory(&mut self, path: &str) -> Result<bool, String> {
        if self.find_by_path(path).is_some() {
            return Ok(false);
        }

        let now = now_ts();
        let id = self.data.next_id;
        self.data.next_id += 1;

        self.data.directories.push(SearchDirectory {
            id: Some(id),
            path: path.to_string(),
            enabled: true,
            include_hidden: false,
            created_at: now,
            last_indexed_at: None,
            file_count: 0,
        });

        Ok(true)
    }
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
