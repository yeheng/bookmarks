-- Create collections table
CREATE TABLE collections (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT DEFAULT '#3b82f6',
    icon TEXT DEFAULT 'folder',
    sort_order INTEGER DEFAULT 0,
    is_default INTEGER DEFAULT 0,
    is_public INTEGER DEFAULT 0,
    parent_id INTEGER REFERENCES collections(id) ON DELETE CASCADE,
    bookmark_count INTEGER DEFAULT 0,
    created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    CONSTRAINT collections_user_name_unique UNIQUE(user_id, name)
);

-- Create indexes
CREATE INDEX idx_collections_user_id ON collections(user_id);
CREATE INDEX idx_collections_parent_id ON collections(parent_id);
CREATE INDEX idx_collections_sort_order ON collections(sort_order);
CREATE INDEX idx_collections_is_default ON collections(is_default);
CREATE INDEX idx_collections_is_public ON collections(is_public);

-- Create trigger to update updated_at timestamp
CREATE TRIGGER update_collections_updated_at BEFORE UPDATE ON collections
    FOR EACH ROW BEGIN
        UPDATE collections SET updated_at = CAST(strftime('%s', 'now') AS INTEGER) WHERE id = NEW.id;
    END;
