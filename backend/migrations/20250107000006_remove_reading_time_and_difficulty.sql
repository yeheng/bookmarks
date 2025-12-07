-- ============================================================
-- 删除不需要的字段
-- 移除 reading_time 和 difficulty_level 字段
-- 创建时间: 2025-01-07
-- ============================================================

-- 删除 reading_time 字段
-- SQLite 不支持 DROP COLUMN，需要重建表
CREATE TABLE resources_new (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    collection_id INTEGER REFERENCES collections(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    url TEXT,
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
    metadata TEXT DEFAULT '{}',
    type TEXT DEFAULT 'link' NOT NULL,
    content TEXT,
    source TEXT,
    mime_type TEXT,
    created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER))
);

-- 迁移数据
INSERT INTO resources_new
SELECT
    id, user_id, collection_id, title, url, description,
    favicon_url, screenshot_url, thumbnail_url,
    is_favorite, is_archived, is_private, is_read,
    visit_count, last_visited, metadata,
    type, content, source, mime_type,
    created_at, updated_at
FROM resources;

-- 删除旧表，重命名新表
DROP TABLE resources;
ALTER TABLE resources_new RENAME TO resources;

-- 重建索引
CREATE INDEX idx_resources_user_id ON resources(user_id);
CREATE INDEX idx_resources_collection_id ON resources(collection_id);
CREATE INDEX idx_resources_is_favorite ON resources(is_favorite);
CREATE INDEX idx_resources_is_archived ON resources(is_archived);
CREATE INDEX idx_resources_is_private ON resources(is_private);
CREATE INDEX idx_resources_is_read ON resources(is_read);
CREATE INDEX idx_resources_created_at ON resources(created_at DESC);
CREATE INDEX idx_resources_last_visited ON resources(last_visited DESC);
CREATE INDEX idx_resources_visit_count ON resources(visit_count DESC);
CREATE INDEX idx_resources_type ON resources(type);

-- 重建 FTS5 索引
DROP TABLE IF EXISTS resources_fts;

CREATE VIRTUAL TABLE resources_fts USING fts5(
    title,           -- 资源标题
    description,     -- 资源描述
    content,         -- 笔记/代码片段内容
    tags,            -- 标签文本
    url,             -- URL（可选）
    tokenize = 'unicode61 remove_diacritics 2'
);

-- 迁移现有数据到新 FTS 索引
INSERT INTO resources_fts (rowid, title, description, content, tags, url)
SELECT
    r.id,
    r.title,
    COALESCE(r.description, ''),
    COALESCE(r.content, ''),
    COALESCE(
        (SELECT GROUP_CONCAT(t.name, ' ')
         FROM resource_tags rt
         JOIN tags t ON rt.tag_id = t.id
         WHERE rt.resource_id = r.id),
        ''
    ) as tags,
    COALESCE(r.url, '')
FROM resources r;