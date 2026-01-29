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

    pub fn initialize(&self) -> Result<()> {
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

            CREATE VIRTUAL TABLE IF NOT EXISTS bookmarks_fts USING fts5(
                title,
                url,
                description,
                tags,
                content=bookmarks,
                content_rowid=id
            );

            CREATE TRIGGER IF NOT EXISTS bookmarks_ai AFTER INSERT ON bookmarks BEGIN
                INSERT INTO bookmarks_fts(rowid, title, url, description, tags)
                VALUES (new.id, new.title, new.url, new.description, new.tags);
            END;

            CREATE TRIGGER IF NOT EXISTS bookmarks_ad AFTER DELETE ON bookmarks BEGIN
                DELETE FROM bookmarks_fts WHERE rowid = old.id;
            END;

            CREATE TRIGGER IF NOT EXISTS bookmarks_au AFTER UPDATE ON bookmarks BEGIN
                UPDATE bookmarks_fts
                SET title = new.title,
                    url = new.url,
                    description = new.description,
                    tags = new.tags
                WHERE rowid = new.id;
            END;

            -- Phase 5: File Search Tables

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

            CREATE VIRTUAL TABLE IF NOT EXISTS files_fts USING fts5(
                name,
                path,
                extension,
                content=indexed_files,
                content_rowid=id
            );

            CREATE TRIGGER IF NOT EXISTS files_ai AFTER INSERT ON indexed_files BEGIN
                INSERT INTO files_fts(rowid, name, path, extension)
                VALUES (new.id, new.name, new.path, new.extension);
            END;

            CREATE TRIGGER IF NOT EXISTS files_ad AFTER DELETE ON indexed_files BEGIN
                DELETE FROM files_fts WHERE rowid = old.id;
            END;

            CREATE TRIGGER IF NOT EXISTS files_au AFTER UPDATE ON indexed_files BEGIN
                UPDATE files_fts
                SET name = new.name,
                    path = new.path,
                    extension = new.extension
                WHERE rowid = new.id;
            END;

            -- Phase 7: Settings Table

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
