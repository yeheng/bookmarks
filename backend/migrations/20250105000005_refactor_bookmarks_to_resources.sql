-- ============================================================
-- 重构迁移: bookmarks → resources
-- 将单一用途的书签系统扩展为通用资源管理系统
-- 创建时间: 2025-01-05
-- ============================================================

-- ============================================================
-- Part 1: 表重命名和字段扩充
-- ============================================================

-- 1.1 重命名核心表
ALTER TABLE bookmarks RENAME TO resources;
ALTER TABLE bookmark_tags RENAME TO resource_tags;

-- 1.2 更新关联表的列名
-- SQLite 不支持直接重命名列，需要重建表
CREATE TABLE resource_tags_new (
    resource_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    PRIMARY KEY (resource_id, tag_id)
);

-- 迁移数据
INSERT INTO resource_tags_new (resource_id, tag_id, created_at)
SELECT bookmark_id, tag_id, created_at FROM resource_tags;

-- 删除旧表，重命名新表
DROP TABLE resource_tags;
ALTER TABLE resource_tags_new RENAME TO resource_tags;

-- 重建索引
CREATE INDEX idx_resource_tags_resource_id ON resource_tags(resource_id);
CREATE INDEX idx_resource_tags_tag_id ON resource_tags(tag_id);

-- 1.3 添加新字段到 resources 表
-- type: 资源类型 (link, note, snippet, file)
ALTER TABLE resources ADD COLUMN type TEXT DEFAULT 'link' NOT NULL;

-- content: 笔记/代码片段内容
ALTER TABLE resources ADD COLUMN content TEXT;

-- source: 文件来源路径
ALTER TABLE resources ADD COLUMN source TEXT;

-- mime_type: 文件 MIME 类型
ALTER TABLE resources ADD COLUMN mime_type TEXT;

-- 1.4 将 url 字段改为可选
-- SQLite 不支持 ALTER COLUMN，需要重建表
CREATE TABLE resources_new (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    collection_id INTEGER REFERENCES collections(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    url TEXT,  -- 从 NOT NULL 改为可选
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
    type TEXT DEFAULT 'link' NOT NULL,
    content TEXT,
    source TEXT,
    mime_type TEXT,
    created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER))
);

-- 迁移数据 (所有现有书签默认为 link 类型)
INSERT INTO resources_new
SELECT
    id, user_id, collection_id, title, url, description,
    favicon_url, screenshot_url, thumbnail_url,
    is_favorite, is_archived, is_private, is_read,
    visit_count, last_visited, reading_time, difficulty_level, metadata,
    'link' as type,  -- 所有现有书签标记为 link 类型
    NULL as content,
    NULL as source,
    NULL as mime_type,
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

-- 新增：资源类型索引（高频查询字段）
CREATE INDEX idx_resources_type ON resources(type);

-- ============================================================
-- Part 2: 资源引用系统
-- ============================================================

-- 2.1 创建资源引用关联表
-- 实现资源间的双向关联，构建知识图谱基础
CREATE TABLE resource_references (
    id INTEGER PRIMARY KEY,
    source_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    target_id INTEGER NOT NULL REFERENCES resources(id) ON DELETE CASCADE,
    type TEXT DEFAULT 'related',  -- 关联类型: related, depends_on, references, etc.
    created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    CONSTRAINT resource_references_unique UNIQUE(source_id, target_id, type)
);

-- 2.2 创建引用表索引
CREATE INDEX idx_resource_references_source_id ON resource_references(source_id);
CREATE INDEX idx_resource_references_target_id ON resource_references(target_id);
CREATE INDEX idx_resource_references_type ON resource_references(type);

-- ============================================================
-- Part 3: 全文搜索索引重建
-- ============================================================

-- 3.1 删除旧的 FTS5 索引
DROP TABLE IF EXISTS bookmarks_fts;

-- 3.2 创建新的 FTS5 索引（包含 content 字段）
CREATE VIRTUAL TABLE resources_fts USING fts5(
    title,           -- 资源标题
    description,     -- 资源描述
    content,         -- 新增：笔记/代码片段内容
    tags,            -- 标签文本
    url,             -- URL（可选）
    tokenize = 'unicode61 remove_diacritics 2'
);

-- 3.3 迁移现有数据到新 FTS 索引
-- 通过聚合标签到一个字段中，简化 FTS 索引
INSERT INTO resources_fts (rowid, title, description, content, tags, url)
SELECT
    r.id,
    r.title,
    COALESCE(r.description, ''),
    COALESCE(r.content, ''),  -- 现有数据的 content 为 NULL
    COALESCE(
        (SELECT GROUP_CONCAT(t.name, ' ')
         FROM resource_tags rt
         JOIN tags t ON rt.tag_id = t.id
         WHERE rt.resource_id = r.id),
        ''
    ) as tags,
    COALESCE(r.url, '')
FROM resources r;

-- ============================================================
-- Part 4: 更新集合表的计数字段名称
-- ============================================================

-- 虽然字段名是 bookmark_count，但语义上仍然正确（集合中的资源数量）
-- 为了保持向后兼容，暂时保留该字段名
-- 未来可以在后续迁移中重命名为 resource_count

-- ============================================================
-- 验证迁移结果
-- ============================================================

-- 查询迁移后的数据统计
-- SELECT
--     type,
--     COUNT(*) as count,
--     COUNT(CASE WHEN url IS NULL THEN 1 END) as url_null_count,
--     COUNT(CASE WHEN content IS NULL THEN 1 END) as content_null_count
-- FROM resources
-- GROUP BY type;

-- ============================================================
-- 注意事项
-- ============================================================

-- 1. 所有现有书签自动标记为 type='link'
-- 2. url 字段从 NOT NULL 改为可选，支持纯笔记类型
-- 3. FTS 索引包含 content 字段，支持笔记内容全文搜索
-- 4. resource_references 表实现资源间关联
-- 5. 迁移完成后，建议运行 VACUUM 清理数据库空间
-- 6. 向后兼容：所有现有数据保持完整，API 需要适配新字段

-- ============================================================
-- 回滚脚本 (仅供参考，实际回滚需要谨慎)
-- ============================================================

-- 回滚时需要：
-- 1. 删除新增的字段和表
-- 2. 重命名表回原名
-- 3. 重建 FTS 索引
-- 实际生产环境建议通过数据库备份恢复
