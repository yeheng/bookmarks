-- ============================================================
-- 合并的 FTS5 全文搜索 Migration
-- 包含 FTS5 虚拟表创建和数据迁移
-- 创建时间: 2025-01-04
-- ============================================================

-- ============================================================
-- FTS5 虚拟表创建
-- 支持中英文混合搜索的分词策略
-- ============================================================

-- 创建 FTS5 虚拟表用于全文搜索
-- 分词策略：
-- - 默认：使用 SQLite FTS5 内置的 unicode61 分词器，存储原始文本
-- - 启用 jieba feature：存储经过 jieba 分词的空格分隔文本
CREATE VIRTUAL TABLE bookmarks_fts USING fts5(
    title,           -- 书签标题（原始文本或 jieba 分词）
    description,     -- 书签描述（原始文本或 jieba 分词）
    tags,            -- 标签文本（空格分隔，原始或 jieba 分词）
    url,             -- URL（原始文本，用于搜索域名）
    tokenize = 'unicode61 remove_diacritics 2'  -- Unicode 分词器，支持中文和英文
);

-- ============================================================
-- 分词器说明
-- ============================================================

-- unicode61: 基于 Unicode 6.1 标准的分词器，能够处理中英文混合文本
-- remove_diacritics 2: 移除变音符号，提高搜索匹配率
-- 
-- 默认模式下，存储原始文本，让 FTS5 的 unicode61 分词器在搜索时处理分词
-- jieba 模式下，存储预分词的空格分隔文本，提供更精确的中文分词

-- ============================================================
-- 数据映射关系
-- ============================================================

-- 使用 bookmarks.id = bookmarks_fts.rowid 来建立映射关系
-- SQLite FTS5 的 rowid 必须与主表的 INTEGER PRIMARY KEY 严格对应
-- 这样可以保证在同一事务中的数据一致性

-- ============================================================
-- FTS5 触发器 (自动同步数据)
-- 确保书签数据变更时自动更新 FTS5 索引
-- ============================================================

-- 插入触发器
CREATE TRIGGER bookmarks_fts_insert AFTER INSERT ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(rowid, title, description, tags, url)
    VALUES (
        NEW.id,
        NEW.title,
        COALESCE(NEW.description, ''),
        COALESCE(
            (SELECT GROUP_CONCAT(t.name, ' ')
             FROM bookmark_tags bt
             JOIN tags t ON bt.tag_id = t.id
             WHERE bt.bookmark_id = NEW.id),
            ''
        ),
        NEW.url
    );
END;

-- 更新触发器
CREATE TRIGGER bookmarks_fts_update AFTER UPDATE ON bookmarks BEGIN
    UPDATE bookmarks_fts SET
        title = NEW.title,
        description = COALESCE(NEW.description, ''),
        tags = COALESCE(
            (SELECT GROUP_CONCAT(t.name, ' ')
             FROM bookmark_tags bt
             JOIN tags t ON bt.tag_id = t.id
             WHERE bt.bookmark_id = NEW.id),
            ''
        ),
        url = NEW.url
    WHERE rowid = NEW.id;
END;

-- 删除触发器
CREATE TRIGGER bookmarks_fts_delete AFTER DELETE ON bookmarks BEGIN
    DELETE FROM bookmarks_fts WHERE rowid = OLD.id;
END;

-- 标签变更触发器 (当书签标签关系变更时更新 FTS)
CREATE TRIGGER bookmarks_fts_tags_update AFTER INSERT ON bookmark_tags BEGIN
    UPDATE bookmarks_fts SET
        tags = COALESCE(
            (SELECT GROUP_CONCAT(t.name, ' ')
             FROM bookmark_tags bt
             JOIN tags t ON bt.tag_id = t.id
             WHERE bt.bookmark_id = NEW.bookmark_id),
            ''
        )
    WHERE rowid = NEW.bookmark_id;
END;

CREATE TRIGGER bookmarks_fts_tags_delete AFTER DELETE ON bookmark_tags BEGIN
    UPDATE bookmarks_fts SET
        tags = COALESCE(
            (SELECT GROUP_CONCAT(t.name, ' ')
             FROM bookmark_tags bt
             JOIN tags t ON bt.tag_id = t.id
             WHERE bt.bookmark_id = OLD.bookmark_id),
            ''
        )
    WHERE rowid = OLD.bookmark_id;
END;

-- ============================================================
-- 分词策略说明和迁移建议
-- ============================================================

-- 分词策略说明：
-- - 默认模式：直接插入原始文本，让 SQLite FTS5 的 unicode61 分词器处理
-- - jieba 模式：需要通过 Rust 程序进行 jieba 分词预处理

-- 推荐的迁移方式：
-- 1. 默认模式：上述 SQL 可以正常工作，SQLite FTS5 会处理分词
-- 2. jieba 模式：使用 rebuild_fts 二进制程序进行分词预处理
--    运行方式：cargo run --bin rebuild_fts
--    该程序会根据当前的 feature 配置自动选择合适的分词策略

-- 性能优化建议：
-- 1. FTS5 索引会增加存储空间开销，但显著提升搜索性能
-- 2. 触发器确保数据一致性，但会增加写入成本
-- 3. 对于大量数据导入，可以临时禁用触发器，批量导入后重建索引
-- 4. 定期使用 'INSERT INTO bookmarks_fts(bookmarks_fts) VALUES('rebuild')' 优化索引

-- 搜索示例：
-- -- 搜索标题和描述
-- SELECT * FROM bookmarks_fts WHERE bookmarks_fts MATCH 'search_term';
-- 
-- -- 搜索特定标签
-- SELECT * FROM bookmarks_fts WHERE bookmarks_fts MATCH 'tags:tag_name';
-- 
-- -- 搜索 URL 域名
-- SELECT * FROM bookmarks_fts WHERE bookmarks_fts MATCH 'url:example.com';

-- ============================================================
-- 注意事项
-- ============================================================

-- 1. FTS5 表的数据自动与 bookmarks 表同步
-- 2. 标签变更会自动更新 FTS 索引中的 tags 字段
-- 3. 搜索性能取决于索引质量和数据量
-- 4. 如启用 jieba 分词，需要使用专门的 rebuild_fts 工具
-- 5. 定期维护 FTS 索引以保持最佳性能