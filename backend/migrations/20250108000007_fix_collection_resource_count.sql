-- ============================================================
-- 修复集合表字段名不一致问题
-- 将 bookmark_count 重命名为 resource_count 以匹配 Rust 模型
-- 创建时间: 2025-01-08
-- ============================================================

-- 重命名 bookmark_count 为 resource_count
-- SQLite 不支持直接重命名列，需要重建表
CREATE TABLE collections_new (
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
    resource_count INTEGER DEFAULT 0,
    created_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    updated_at INTEGER DEFAULT (CAST(strftime('%s', 'now') AS INTEGER)),
    CONSTRAINT collections_user_name_unique UNIQUE(user_id, name)
);

-- 迁移数据
INSERT INTO collections_new
SELECT
    id, user_id, name, description, color, icon, sort_order,
    is_default, is_public, parent_id, bookmark_count, created_at, updated_at
FROM collections;

-- 删除旧表，重命名新表
DROP TABLE collections;
ALTER TABLE collections_new RENAME TO collections;

-- 重建索引
CREATE INDEX idx_collections_user_id ON collections(user_id);
CREATE INDEX idx_collections_parent_id ON collections(parent_id);
CREATE INDEX idx_collections_sort_order ON collections(sort_order);
CREATE INDEX idx_collections_is_default ON collections(is_default);
CREATE INDEX idx_collections_is_public ON collections(is_public);