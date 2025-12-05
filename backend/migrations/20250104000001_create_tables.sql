-- ============================================================
-- 用户表 (users)
-- ============================================================
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    avatar_url TEXT,
    is_active INTEGER DEFAULT 1,
    email_verified INTEGER DEFAULT 0,
    email_verification_token TEXT,
    password_reset_token TEXT,
    password_reset_expires_at INTEGER,
    last_login_at INTEGER,
    created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER))
);

-- ============================================================
-- 集合表 (collections)
-- ============================================================
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

-- ============================================================
-- 标签表 (tags)
-- ============================================================
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

-- ============================================================
-- 书签表 (bookmarks)
-- ============================================================
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

-- ============================================================
-- 书签标签关联表 (bookmark_tags)
-- ============================================================
CREATE TABLE bookmark_tags (
    bookmark_id INTEGER NOT NULL REFERENCES bookmarks(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    PRIMARY KEY (bookmark_id, tag_id)
);

-- ============================================================
-- 基础索引 (必需的核心索引)
-- 这些索引在表创建时立即建立，确保基本查询性能
-- ============================================================

-- 用户表基础索引
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_active ON users(is_active);
CREATE INDEX idx_users_email_verified ON users(email_verified);

-- 集合表基础索引
CREATE INDEX idx_collections_user_id ON collections(user_id);
CREATE INDEX idx_collections_parent_id ON collections(parent_id);
CREATE INDEX idx_collections_sort_order ON collections(sort_order);
CREATE INDEX idx_collections_is_default ON collections(is_default);
CREATE INDEX idx_collections_is_public ON collections(is_public);

-- 标签表基础索引
CREATE INDEX idx_tags_user_id ON tags(user_id);
CREATE INDEX idx_tags_name ON tags(name);
CREATE INDEX idx_tags_usage_count ON tags(usage_count DESC);
CREATE INDEX idx_tags_created_at ON tags(created_at DESC);

-- 书签表基础索引
CREATE INDEX idx_bookmarks_user_id ON bookmarks(user_id);
CREATE INDEX idx_bookmarks_collection_id ON bookmarks(collection_id);
CREATE INDEX idx_bookmarks_is_favorite ON bookmarks(is_favorite);
CREATE INDEX idx_bookmarks_is_archived ON bookmarks(is_archived);
CREATE INDEX idx_bookmarks_is_private ON bookmarks(is_private);
CREATE INDEX idx_bookmarks_is_read ON bookmarks(is_read);
CREATE INDEX idx_bookmarks_created_at ON bookmarks(created_at DESC);
CREATE INDEX idx_bookmarks_last_visited ON bookmarks(last_visited DESC);
CREATE INDEX idx_bookmarks_visit_count ON bookmarks(visit_count DESC);

-- 书签标签关联表基础索引
CREATE INDEX idx_bookmark_tags_bookmark_id ON bookmark_tags(bookmark_id);
CREATE INDEX idx_bookmark_tags_tag_id ON bookmark_tags(tag_id);
