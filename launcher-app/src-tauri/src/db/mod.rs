use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Database { conn })
    }

    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Ok(Database { conn })
    }

    pub fn initialize(&self) -> Result<()> {
        // Enable WAL mode for better concurrent read/write performance
        // WAL mode allows readers to not block writers and vice versa
        self.conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA foreign_keys=ON;
             PRAGMA busy_timeout=5000;
             PRAGMA synchronous=NORMAL;",
        )?;

        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS bookmarks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                url TEXT NOT NULL UNIQUE,
                description TEXT,
                favicon_url TEXT,
                tags TEXT,
                source TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                last_accessed INTEGER
            );

            CREATE TABLE IF NOT EXISTS usage_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bookmark_id INTEGER NOT NULL,
                accessed_at INTEGER NOT NULL,
                FOREIGN KEY (bookmark_id) REFERENCES bookmarks(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_bookmarks_url ON bookmarks(url);
            CREATE INDEX IF NOT EXISTS idx_bookmarks_source ON bookmarks(source);
            CREATE INDEX IF NOT EXISTS idx_bookmarks_last_accessed ON bookmarks(last_accessed);
            CREATE INDEX IF NOT EXISTS idx_usage_history_bookmark_id ON usage_history(bookmark_id);
            CREATE INDEX IF NOT EXISTS idx_usage_history_accessed_at ON usage_history(accessed_at);

            -- Note: FTS5 virtual tables removed - using Tantivy for full-text search

            -- File Search Tables

            CREATE TABLE IF NOT EXISTS search_directories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                enabled INTEGER NOT NULL DEFAULT 1,
                include_hidden INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                last_indexed_at INTEGER,
                file_count INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS indexed_files (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                extension TEXT,
                size INTEGER NOT NULL,
                modified_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                indexed_at INTEGER NOT NULL,
                directory_id INTEGER NOT NULL,
                FOREIGN KEY (directory_id) REFERENCES search_directories(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS file_usage_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id INTEGER NOT NULL,
                accessed_at INTEGER NOT NULL,
                FOREIGN KEY (file_id) REFERENCES indexed_files(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_indexed_files_path ON indexed_files(path);
            CREATE INDEX IF NOT EXISTS idx_indexed_files_name ON indexed_files(name);
            CREATE INDEX IF NOT EXISTS idx_indexed_files_extension ON indexed_files(extension);
            CREATE INDEX IF NOT EXISTS idx_indexed_files_directory_id ON indexed_files(directory_id);
            CREATE INDEX IF NOT EXISTS idx_indexed_files_modified_at ON indexed_files(modified_at);
            CREATE INDEX IF NOT EXISTS idx_file_usage_history_file_id ON file_usage_history(file_id);
            CREATE INDEX IF NOT EXISTS idx_file_usage_history_accessed_at ON file_usage_history(accessed_at);

            -- Note: FTS5 virtual tables for files removed - using Tantivy for full-text search

            -- Settings Table

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            );
            "#,
        )?;

        Ok(())
    }

    pub fn get_connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn setup_db() -> Database {
        let db = Database::new_in_memory().unwrap();
        db.initialize().unwrap();
        db
    }

    fn now() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    #[test]
    fn test_database_initialization() {
        let db = setup_db();
        let conn = db.get_connection();

        // Verify all tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        assert!(tables.contains(&"bookmarks".to_string()));
        assert!(tables.contains(&"indexed_files".to_string()));
        assert!(tables.contains(&"settings".to_string()));
        assert!(tables.contains(&"search_directories".to_string()));
    }

    #[test]
    fn test_bookmark_crud() {
        let db = setup_db();
        let conn = db.get_connection();
        let ts = now();

        // Create
        conn.execute(
            "INSERT INTO bookmarks (title, url, source, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["Test Bookmark", "https://example.com", "manual", ts, ts],
        ).unwrap();

        let id = conn.last_insert_rowid();

        // Read
        let (title, url): (String, String) = conn
            .query_row(
                "SELECT title, url FROM bookmarks WHERE id = ?1",
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(title, "Test Bookmark");
        assert_eq!(url, "https://example.com");

        // Update
        conn.execute(
            "UPDATE bookmarks SET title = ?1 WHERE id = ?2",
            rusqlite::params!["Updated Bookmark", id],
        )
        .unwrap();

        let title: String = conn
            .query_row("SELECT title FROM bookmarks WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(title, "Updated Bookmark");

        // Delete
        conn.execute("DELETE FROM bookmarks WHERE id = ?1", [id])
            .unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM bookmarks WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_settings_crud() {
        let db = setup_db();
        let conn = db.get_connection();
        let ts = now();

        // Create
        conn.execute(
            "INSERT INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            rusqlite::params!["theme.mode", "dark", ts],
        )
        .unwrap();

        // Read
        let value: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                ["theme.mode"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "dark");

        // Update (upsert)
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value, updated_at) VALUES (?1, ?2, ?3)",
            rusqlite::params!["theme.mode", "light", ts],
        )
        .unwrap();

        let value: String = conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                ["theme.mode"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(value, "light");

        // Delete
        conn.execute("DELETE FROM settings WHERE key = ?1", ["theme.mode"])
            .unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM settings WHERE key = ?1",
                ["theme.mode"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_file_indexing() {
        let db = setup_db();
        let conn = db.get_connection();
        let ts = now();

        // Create directory
        conn.execute(
            "INSERT INTO search_directories (path, enabled, include_hidden, created_at, file_count) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["/test/dir", 1, 0, ts, 0],
        ).unwrap();
        let dir_id = conn.last_insert_rowid();

        // Index files
        conn.execute(
            "INSERT INTO indexed_files (path, name, extension, size, modified_at, created_at, indexed_at, directory_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params!["/test/dir/document.pdf", "document.pdf", "pdf", 1024, ts, ts, ts, dir_id],
        ).unwrap();

        conn.execute(
            "INSERT INTO indexed_files (path, name, extension, size, modified_at, created_at, indexed_at, directory_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params!["/test/dir/image.png", "image.png", "png", 2048, ts, ts, ts, dir_id],
        ).unwrap();

        // Search by extension
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM indexed_files WHERE extension = ?1",
                ["pdf"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        // Verify file exists
        let name: String = conn
            .query_row(
                "SELECT name FROM indexed_files WHERE extension = ?1",
                ["pdf"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(name, "document.pdf");
    }

    #[test]
    fn test_usage_history_tracking() {
        let db = setup_db();
        let conn = db.get_connection();
        let ts = now();

        // Create bookmark
        conn.execute(
            "INSERT INTO bookmarks (title, url, source, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params!["Test", "https://test.com", "manual", ts, ts],
        ).unwrap();
        let bookmark_id = conn.last_insert_rowid();

        // Record usage
        for i in 0..3 {
            conn.execute(
                "INSERT INTO usage_history (bookmark_id, accessed_at) VALUES (?1, ?2)",
                rusqlite::params![bookmark_id, ts + i],
            )
            .unwrap();
        }

        // Count usage
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM usage_history WHERE bookmark_id = ?1",
                [bookmark_id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3);
    }
}
