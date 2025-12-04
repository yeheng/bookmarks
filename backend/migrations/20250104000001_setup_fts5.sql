-- Create FTS5 virtual table for full-text search
-- 使用 jieba 分词后的空格分隔文本作为输入
CREATE VIRTUAL TABLE bookmarks_fts USING fts5(
    title,           -- 书签标题（经过 jieba 分词）
    description,     -- 书签描述（经过 jieba 分词）
    tags,            -- 标签文本（经过 jieba 分词）
    url,             -- URL（原始文本，用于搜索域名）
    tokenize = 'unicode61 remove_diacritics 2'  -- 基础 Unicode 分词器
);

-- 注意：我们使用 bookmarks.id = bookmarks_fts.rowid 来建立映射关系
-- SQLite FTS5 的 rowid 必须与主表的 INTEGER PRIMARY KEY 严格对应
-- 这样可以保证在同一事务中的数据一致性
