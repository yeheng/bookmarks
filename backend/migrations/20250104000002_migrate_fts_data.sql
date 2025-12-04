-- 数据迁移脚本：将现有书签数据迁移到 FTS5 索引
-- 这个脚本需要在部署新版本后手动执行一次

-- 首先清空 FTS5 表（如果有数据）
DELETE FROM bookmarks_fts;

-- 将所有现有书签数据插入 FTS5 索引
-- 注意：这里需要手动对每个字段进行 jieba 分词
-- 由于 SQL 中无法直接调用 jieba，这个脚本只是模板
-- 实际迁移需要通过 Rust 程序完成（见下方说明）

-- 临时说明：此 SQL 脚本仅用于记录，实际迁移应使用 rebuild_fts 二进制程序
-- 运行方式：cargo run --bin rebuild_fts

-- 如果要手动迁移少量数据，可以使用以下 SQL（但分词效果会很差）：
-- INSERT INTO bookmarks_fts (rowid, title, description, tags, url)
-- SELECT
--     b.id,
--     b.title,
--     COALESCE(b.description, ''),
--     COALESCE(
--         (SELECT GROUP_CONCAT(t.name, ' ')
--          FROM bookmark_tags bt
--          JOIN tags t ON bt.tag_id = t.id
--          WHERE bt.bookmark_id = b.id),
--         ''
--     ),
--     b.url
-- FROM bookmarks b;

-- 警告：上述 SQL 未经过 jieba 分词，搜索效果会很差
-- 强烈建议使用专门的迁移程序：cargo run --bin rebuild_fts
