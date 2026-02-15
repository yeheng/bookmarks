use crate::models::bookmark::Bookmark;
use crate::store::json_store::JsonStore;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookmarkStoreData {
    pub next_id: i64,
    pub bookmarks: Vec<Bookmark>,
}

impl Default for BookmarkStoreData {
    fn default() -> Self {
        Self {
            next_id: 1,
            bookmarks: Vec::new(),
        }
    }
}

pub struct BookmarkStore {
    store: JsonStore,
    data: BookmarkStoreData,
}

impl BookmarkStore {
    pub fn new(path: PathBuf) -> Self {
        let store = JsonStore::new(path);
        let data: BookmarkStoreData = store.load();
        Self { store, data }
    }

    pub fn save(&self) -> Result<(), String> {
        self.store.save(&self.data)
    }

    pub fn bookmarks(&self) -> &[Bookmark] {
        &self.data.bookmarks
    }

    pub fn bookmarks_mut(&mut self) -> &mut Vec<Bookmark> {
        &mut self.data.bookmarks
    }

    pub fn count(&self) -> usize {
        self.data.bookmarks.len()
    }

    pub fn find_by_id(&self, id: i64) -> Option<&Bookmark> {
        self.data.bookmarks.iter().find(|b| b.id == Some(id))
    }

    pub fn find_by_url(&self, url: &str) -> Option<&Bookmark> {
        self.data.bookmarks.iter().find(|b| b.url == url)
    }

    pub fn add(
        &mut self,
        title: String,
        url: String,
        description: Option<String>,
        tags: Option<String>,
        source: String,
    ) -> Result<i64, String> {
        // Check for duplicate URL
        if self.find_by_url(&url).is_some() {
            return Err(format!("Bookmark with URL '{}' already exists", url));
        }

        let now = now_ts();
        let id = self.data.next_id;
        self.data.next_id += 1;

        let bookmark = Bookmark {
            id: Some(id),
            title,
            url,
            description,
            favicon_url: None,
            tags,
            source,
            created_at: now,
            updated_at: now,
            last_accessed: None,
        };

        self.data.bookmarks.push(bookmark);
        self.save()?;
        Ok(id)
    }

    pub fn update(
        &mut self,
        id: i64,
        title: String,
        url: String,
        description: Option<String>,
        tags: Option<String>,
    ) -> Result<(), String> {
        let bookmark = self
            .data
            .bookmarks
            .iter_mut()
            .find(|b| b.id == Some(id))
            .ok_or("Bookmark not found")?;

        bookmark.title = title;
        bookmark.url = url;
        bookmark.description = description;
        bookmark.tags = tags;
        bookmark.updated_at = now_ts();

        self.save()
    }

    pub fn delete(&mut self, id: i64) -> Result<(), String> {
        let len_before = self.data.bookmarks.len();
        self.data.bookmarks.retain(|b| b.id != Some(id));

        if self.data.bookmarks.len() == len_before {
            return Err("Bookmark not found".to_string());
        }

        self.save()
    }

    pub fn update_last_accessed(&mut self, id: i64, timestamp: i64) -> Result<(), String> {
        let bookmark = self
            .data
            .bookmarks
            .iter_mut()
            .find(|b| b.id == Some(id))
            .ok_or("Bookmark not found")?;

        bookmark.last_accessed = Some(timestamp);
        self.save()
    }

    /// Add a bookmark imported from a browser. Returns Ok(true) if imported, Ok(false) if skipped.
    pub fn import_bookmark(
        &mut self,
        title: &str,
        url: &str,
        source: &str,
    ) -> Result<bool, String> {
        if self.find_by_url(url).is_some() {
            return Ok(false);
        }

        let now = now_ts();
        let id = self.data.next_id;
        self.data.next_id += 1;

        self.data.bookmarks.push(Bookmark {
            id: Some(id),
            title: title.to_string(),
            url: url.to_string(),
            description: None,
            favicon_url: None,
            tags: None,
            source: source.to_string(),
            created_at: now,
            updated_at: now,
            last_accessed: None,
        });

        // Don't save after every import — caller should batch-save
        Ok(true)
    }

    /// Get bookmark data tuples for Tantivy index rebuild.
    pub fn get_index_data(&self) -> Vec<(i64, String, String, Option<String>, Option<String>, Option<i64>, i64, i64)> {
        self.data
            .bookmarks
            .iter()
            .filter_map(|b| {
                Some((
                    b.id?,
                    b.title.clone(),
                    b.url.clone(),
                    b.description.clone(),
                    b.tags.clone(),
                    b.last_accessed,
                    b.created_at,
                    b.updated_at,
                ))
            })
            .collect()
    }

    /// Get recently imported bookmarks (by source and created_at threshold) for incremental indexing.
    pub fn get_recently_imported(&self, source: &str, since: i64) -> Vec<(i64, String, String, Option<String>, Option<String>, Option<i64>, i64, i64)> {
        self.data
            .bookmarks
            .iter()
            .filter(|b| b.source == source && b.created_at >= since)
            .filter_map(|b| {
                Some((
                    b.id?,
                    b.title.clone(),
                    b.url.clone(),
                    b.description.clone(),
                    b.tags.clone(),
                    b.last_accessed,
                    b.created_at,
                    b.updated_at,
                ))
            })
            .collect()
    }
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
