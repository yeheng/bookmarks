-- Fix foreign key constraint and add missing indexes

-- Fix bookmark_stats table to use INTEGER instead of BLOB for user_id
DROP TABLE IF EXISTS bookmark_stats;

CREATE TABLE bookmark_stats (
    user_id INTEGER PRIMARY KEY,
    total_bookmarks INTEGER DEFAULT 0,
    favorite_count INTEGER DEFAULT 0,
    archived_count INTEGER DEFAULT 0,
    unread_count INTEGER DEFAULT 0,
    total_visits INTEGER DEFAULT 0,
    last_calculated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Recreate stats update triggers with optimized performance
DROP TRIGGER IF EXISTS update_bookmark_stats_after_insert;
CREATE TRIGGER update_bookmark_stats_after_insert
AFTER INSERT ON bookmarks BEGIN
    INSERT OR REPLACE INTO bookmark_stats (user_id, total_bookmarks, favorite_count, unread_count, total_visits, last_calculated_at)
    VALUES (
        NEW.user_id,
        COALESCE((SELECT total_bookmarks FROM bookmark_stats WHERE user_id = NEW.user_id), 0) + 1,
        COALESCE((SELECT favorite_count FROM bookmark_stats WHERE user_id = NEW.user_id), 0) + NEW.is_favorite,
        COALESCE((SELECT unread_count FROM bookmark_stats WHERE user_id = NEW.user_id), 0) + (1 - NEW.is_read),
        COALESCE((SELECT total_visits FROM bookmark_stats WHERE user_id = NEW.user_id), 0) + NEW.visit_count,
        CURRENT_TIMESTAMP
    );
END;

DROP TRIGGER IF EXISTS update_bookmark_stats_after_delete;
CREATE TRIGGER update_bookmark_stats_after_delete
AFTER DELETE ON bookmarks BEGIN
    UPDATE bookmark_stats SET
        total_bookmarks = total_bookmarks - 1,
        favorite_count = favorite_count - OLD.is_favorite,
        unread_count = unread_count - (1 - OLD.is_read),
        total_visits = total_visits - OLD.visit_count,
        last_calculated_at = CURRENT_TIMESTAMP
    WHERE user_id = OLD.user_id;
END;

DROP TRIGGER IF EXISTS update_bookmark_stats_after_update;
CREATE TRIGGER update_bookmark_stats_after_update
AFTER UPDATE ON bookmarks BEGIN
    UPDATE bookmark_stats SET
        favorite_count = favorite_count + (NEW.is_favorite - OLD.is_favorite),
        unread_count = unread_count + (OLD.is_read - NEW.is_read),
        total_visits = total_visits + (NEW.visit_count - OLD.visit_count),
        last_calculated_at = CURRENT_TIMESTAMP
    WHERE user_id = NEW.user_id;
END;

-- Add missing indexes for better performance
CREATE INDEX idx_bookmarks_url ON bookmarks(url);
CREATE INDEX idx_collections_name_search ON collections(name);
CREATE INDEX idx_tags_name_search ON tags(name);