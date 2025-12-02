-- Create bookmarks table
CREATE TABLE bookmarks (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(6)))),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    collection_id TEXT REFERENCES collections(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    description TEXT,
    favicon_url TEXT,
    screenshot_url TEXT,
    thumbnail_url TEXT,
    is_favorite INTEGER DEFAULT 0,
    is_archived INTEGER DEFAULT 0,
    is_private INTEGER DEFAULT 0,
    is_read INTEGER DEFAULT 0,
    visit_count INTEGER DEFAULT 0,
    last_visited DATETIME,
    reading_time INTEGER,
    difficulty_level INTEGER CHECK (difficulty_level BETWEEN 1 AND 5),
    metadata TEXT DEFAULT '{}',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes
CREATE INDEX idx_bookmarks_user_id ON bookmarks(user_id);
CREATE INDEX idx_bookmarks_collection_id ON bookmarks(collection_id);
CREATE INDEX idx_bookmarks_is_favorite ON bookmarks(is_favorite);
CREATE INDEX idx_bookmarks_is_archived ON bookmarks(is_archived);
CREATE INDEX idx_bookmarks_is_private ON bookmarks(is_private);
CREATE INDEX idx_bookmarks_is_read ON bookmarks(is_read);
CREATE INDEX idx_bookmarks_created_at ON bookmarks(created_at DESC);
CREATE INDEX idx_bookmarks_last_visited ON bookmarks(last_visited DESC);
CREATE INDEX idx_bookmarks_visit_count ON bookmarks(visit_count DESC);

-- Full-text search index (SQLite FTS5)
CREATE VIRTUAL TABLE bookmarks_fts USING fts5(
    title, 
    description,
    content='bookmarks',
    content_rowid='rowid'
);

-- FTS triggers
CREATE TRIGGER bookmarks_fts_insert AFTER INSERT ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(rowid, title, description) 
    VALUES (new.rowid, new.title, new.description);
END;

CREATE TRIGGER bookmarks_fts_delete AFTER DELETE ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(bookmarks_fts, rowid, title, description) 
    VALUES ('delete', old.rowid, old.title, old.description);
END;

CREATE TRIGGER bookmarks_fts_update AFTER UPDATE ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(bookmarks_fts, rowid, title, description) 
    VALUES ('delete', old.rowid, old.title, old.description);
    INSERT INTO bookmarks_fts(rowid, title, description) 
    VALUES (new.rowid, new.title, new.description);
END;

-- Create trigger to update updated_at timestamp
CREATE TRIGGER update_bookmarks_updated_at BEFORE UPDATE ON bookmarks
    FOR EACH ROW BEGIN
        UPDATE bookmarks SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;