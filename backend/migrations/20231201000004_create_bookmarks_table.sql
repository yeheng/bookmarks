-- Create bookmarks table
CREATE TABLE bookmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    collection_id UUID REFERENCES collections(id) ON DELETE SET NULL,
    title VARCHAR(255) NOT NULL,
    url VARCHAR(2048) NOT NULL,
    description TEXT,
    favicon_url VARCHAR(2048),
    screenshot_url VARCHAR(2048),
    thumbnail_url VARCHAR(2048),
    is_favorite BOOLEAN DEFAULT FALSE,
    is_archived BOOLEAN DEFAULT FALSE,
    is_private BOOLEAN DEFAULT FALSE,
    is_read BOOLEAN DEFAULT FALSE,
    visit_count INTEGER DEFAULT 0,
    last_visited TIMESTAMP WITH TIME ZONE,
    reading_time INTEGER,
    difficulty_level INTEGER CHECK (difficulty_level BETWEEN 1 AND 5),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
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

-- Full-text search index
CREATE INDEX bookmarks_search_idx ON bookmarks USING GIN (
    to_tsvector('english', title || ' ' || COALESCE(description, ''))
);

-- URL domain index for statistics
CREATE INDEX bookmarks_domain_idx ON bookmarks (regexp_replace(url, '^https?://([^/]+).*', '\1'));

-- Create trigger to update updated_at timestamp
CREATE TRIGGER update_bookmarks_updated_at BEFORE UPDATE ON bookmarks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();