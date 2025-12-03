-- Create tags table
CREATE TABLE tags (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    color TEXT DEFAULT '#64748b',
    description TEXT,
    usage_count INTEGER DEFAULT 0,
    created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),

    CONSTRAINT tags_user_name_unique UNIQUE(user_id, name)
);

-- Create indexes
CREATE INDEX idx_tags_user_id ON tags(user_id);
CREATE INDEX idx_tags_name ON tags(name);
CREATE INDEX idx_tags_usage_count ON tags(usage_count DESC);
CREATE INDEX idx_tags_created_at ON tags(created_at DESC);

-- Create trigger to update updated_at timestamp
CREATE TRIGGER update_tags_updated_at BEFORE UPDATE ON tags
    FOR EACH ROW BEGIN
        UPDATE tags SET updated_at = CAST(strftime('%s', 'now') AS INTEGER) WHERE id = NEW.id;
    END;
