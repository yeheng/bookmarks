-- Create bookmark_tags junction table
CREATE TABLE bookmark_tags (
    bookmark_id INTEGER NOT NULL REFERENCES bookmarks(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),

    PRIMARY KEY (bookmark_id, tag_id)
);

-- Create indexes
CREATE INDEX idx_bookmark_tags_bookmark_id ON bookmark_tags(bookmark_id);
CREATE INDEX idx_bookmark_tags_tag_id ON bookmark_tags(tag_id);
