-- 优化索引设计 - 移除过度设计的复合索引
--
-- 原则:
-- 1. 索引不是免费的 - 每个索引都增加写入成本和存储空间
-- 2. 索引应该基于真实查询模式,而不是臆想的性能问题
-- 3. SQLite 的查询优化器可能用不到过于复杂的复合索引
-- 4. 先实现功能,用真实数据测试,再针对性优化
--
-- Linus 说: "Premature optimization is the root of all evil"
-- 只为真正的性能瓶颈添加索引

-- ============================================================
-- 移除过度设计的复合索引
-- ============================================================

-- collections 表 - 删除从未使用的三列复合索引
-- 保留的索引已经足够: idx_collections_user_id, idx_collections_parent_id
DROP INDEX IF EXISTS idx_collections_user_parent_sort;

-- bookmarks 表 - 删除过度设计的三列复合索引
-- 这些索引基于"臆想"的查询模式,实际查询很少同时用到三列
-- 保留的单列/双列索引已经足够应对当前查询需求
DROP INDEX IF EXISTS idx_bookmarks_user_collection_created;
DROP INDEX IF EXISTS idx_bookmarks_user_favorite_created;
DROP INDEX IF EXISTS idx_bookmarks_user_archived_created;
DROP INDEX IF EXISTS idx_bookmarks_user_read_created;

-- ============================================================
-- 保留的核心索引 (这些是真正需要的)
-- ============================================================

-- 已存在且必需的索引:
-- 1. idx_bookmarks_user_id - 按用户查询书签 (最常用)
-- 2. idx_bookmarks_collection_id - 按集合查询书签
-- 3. idx_bookmarks_url - URL 去重检查
-- 4. idx_bookmarks_created_at - 按时间排序
-- 5. idx_bookmark_tags_bookmark_id - 标签关联查询
-- 6. idx_bookmark_tags_tag_id - 反向标签查询
-- 7. idx_tags_user_name - 标签名查询和去重

-- ============================================================
-- 性能优化建议
-- ============================================================

-- 如果将来发现特定查询慢,应该:
-- 1. 使用 EXPLAIN QUERY PLAN 分析查询
-- 2. 确认查询频率和数据规模
-- 3. 针对性添加索引并测试效果
-- 4. 监控索引使用情况 (SQLite 的 index_info)
--
-- 不要"猜测"需要什么索引,用数据说话
