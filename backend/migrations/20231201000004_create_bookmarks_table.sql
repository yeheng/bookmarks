-- Create bookmarks table
CREATE TABLE bookmarks (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    collection_id INTEGER REFERENCES collections(id) ON DELETE SET NULL,
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
    last_visited INTEGER,
    reading_time INTEGER,
    difficulty_level INTEGER CHECK (difficulty_level BETWEEN 1 AND 5),
    metadata TEXT DEFAULT '{}',
    created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER))
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
