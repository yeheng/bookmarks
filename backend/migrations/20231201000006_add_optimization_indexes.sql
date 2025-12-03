-- Add optimization indexes for better performance

-- Users table composite indexes
CREATE INDEX idx_users_active_email_verified ON users(is_active, email_verified);
CREATE INDEX idx_users_last_login_desc ON users(last_login_at DESC);

-- Collections table composite indexes
CREATE INDEX idx_collections_user_parent_sort ON collections(user_id, parent_id, sort_order);
CREATE INDEX idx_collections_user_public_default ON collections(user_id, is_public, is_default);

-- Bookmarks table composite indexes (key optimizations)
CREATE INDEX idx_bookmarks_user_collection_created ON bookmarks(user_id, collection_id, created_at DESC);
CREATE INDEX idx_bookmarks_user_favorite_created ON bookmarks(user_id, is_favorite, created_at DESC);
CREATE INDEX idx_bookmarks_user_archived_created ON bookmarks(user_id, is_archived, created_at DESC);
CREATE INDEX idx_bookmarks_user_read_visited ON bookmarks(user_id, is_read, last_visited DESC);

-- Tags table composite indexes
CREATE INDEX idx_tags_user_usage_created ON tags(user_id, usage_count DESC, created_at DESC);

-- Bookmark_tags junction table composite indexes
CREATE INDEX idx_bookmark_tags_bookmark_created ON bookmark_tags(bookmark_id, created_at DESC);
CREATE INDEX idx_bookmark_tags_tag_created ON bookmark_tags(tag_id, created_at DESC);

-- Partial indexes for common queries
CREATE INDEX idx_bookmarks_active_users ON bookmarks(user_id, created_at DESC)
WHERE is_archived = 0;

CREATE INDEX idx_bookmarks_favorites ON bookmarks(user_id, created_at DESC)
WHERE is_favorite = 1;

CREATE INDEX idx_bookmarks_unread ON bookmarks(user_id, created_at DESC)
WHERE is_read = 0;

-- Create bookmark statistics table for performance
CREATE TABLE bookmark_stats (
    user_id BLOB PRIMARY KEY,
    total_bookmarks INTEGER DEFAULT 0,
    favorite_count INTEGER DEFAULT 0,
    archived_count INTEGER DEFAULT 0,
    unread_count INTEGER DEFAULT 0,
    total_visits INTEGER DEFAULT 0,
    last_calculated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create stats update triggers
CREATE TRIGGER update_bookmark_stats_after_insert
AFTER INSERT ON bookmarks BEGIN
    INSERT OR REPLACE INTO bookmark_stats (user_id, total_bookmarks, favorite_count, unread_count, total_visits, last_calculated_at)
    SELECT
        NEW.user_id,
        COALESCE((SELECT total_bookmarks FROM bookmark_stats WHERE user_id = NEW.user_id), 0) + 1,
        COALESCE((SELECT favorite_count FROM bookmark_stats WHERE user_id = NEW.user_id), 0) + NEW.is_favorite,
        COALESCE((SELECT unread_count FROM bookmark_stats WHERE user_id = NEW.user_id), 0) + (1 - NEW.is_read),
        COALESCE((SELECT total_visits FROM bookmark_stats WHERE user_id = NEW.user_id), 0) + NEW.visit_count,
        CURRENT_TIMESTAMP
    WHERE 1;
END;

CREATE TRIGGER update_bookmark_stats_after_delete
AFTER DELETE ON bookmarks BEGIN
    UPDATE bookmark_stats SET
        total_bookmarks = total_bookmarks - 1,
        favorite_count = favorite_count - OLD.is_favorite,
        unread_count = unread_count - (1 - OLD.is_read),
        last_calculated_at = CURRENT_TIMESTAMP
    WHERE user_id = OLD.user_id;
END;

CREATE TRIGGER update_bookmark_stats_after_update
AFTER UPDATE ON bookmarks BEGIN
    UPDATE bookmark_stats SET
        favorite_count = favorite_count + (NEW.is_favorite - OLD.is_favorite),
        unread_count = unread_count + (OLD.is_read - NEW.is_read),
        total_visits = total_visits + (NEW.visit_count - OLD.visit_count),
        last_calculated_at = CURRENT_TIMESTAMP
    WHERE user_id = NEW.user_id;
END;
