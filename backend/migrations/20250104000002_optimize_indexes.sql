-- ============================================================
-- 合并的索引优化 Migration
-- 包含所有性能优化索引，移除过度设计
-- 创建时间: 2025-01-04
-- ============================================================

-- ============================================================
-- 性能优化索引 (Performance Optimization Indexes)
-- 基于真实查询模式设计的复合索引
-- ============================================================

-- 用户表复合索引
CREATE INDEX idx_users_active_email_verified ON users(is_active, email_verified);
CREATE INDEX idx_users_last_login_desc ON users(last_login_at DESC);

-- 集合表复合索引
CREATE INDEX idx_collections_user_public_default ON collections(user_id, is_public, is_default);

-- 标签表复合索引
CREATE INDEX idx_tags_user_usage_created ON tags(user_id, usage_count DESC, created_at DESC);

-- 书签表复合索引 (保留有用的)
CREATE INDEX idx_bookmarks_user_read_visited ON bookmarks(user_id, is_read, last_visited DESC);

-- 书签标签关联表复合索引
CREATE INDEX idx_bookmark_tags_bookmark_created ON bookmark_tags(bookmark_id, created_at DESC);
CREATE INDEX idx_bookmark_tags_tag_created ON bookmark_tags(tag_id, created_at DESC);

-- ============================================================
-- 部分索引 (Partial Indexes)
-- 针对常见查询场景的优化
-- ============================================================

-- 活跃用户书签索引 (排除已归档)
CREATE INDEX idx_bookmarks_active_users ON bookmarks(user_id, created_at DESC)
WHERE is_archived = 0;

-- 收藏书签索引
CREATE INDEX idx_bookmarks_favorites ON bookmarks(user_id, created_at DESC)
WHERE is_favorite = 1;

-- 未读书签索引
CREATE INDEX idx_bookmarks_unread ON bookmarks(user_id, created_at DESC)
WHERE is_read = 0;

-- ============================================================
-- 搜索相关索引
-- 支持模糊搜索和去重检查
-- ============================================================

-- URL 去重索引
CREATE INDEX idx_bookmarks_url ON bookmarks(url);

-- 集合名称搜索索引
CREATE INDEX idx_collections_name_search ON collections(name);

-- 标签名称搜索索引
CREATE INDEX idx_tags_name_search ON tags(name);

-- ============================================================
-- 已移除的过度设计索引说明
-- 以下索引因过度设计或使用率低而被移除
-- ============================================================

-- 移除的索引列表 (不执行，仅作记录):
-- 1. idx_collections_user_parent_sort - 三列复合索引，查询模式不匹配
-- 2. idx_bookmarks_user_collection_created - 过度设计，单列索引已足够
-- 3. idx_bookmarks_user_favorite_created - 过度设计，部分索引更高效
-- 4. idx_bookmarks_user_archived_created - 过度设计，部分索引已覆盖
-- 5. idx_bookmarks_user_read_created - 替换为更实用的 user_read_visited

-- ============================================================
-- 性能优化原则
-- ============================================================

-- 1. 索引不是免费的 - 每个索引都增加写入成本和存储空间
-- 2. 索引基于真实查询模式，而非臆想的性能问题
-- 3. SQLite 查询优化器可能用不到过于复杂的复合索引
-- 4. 先实现功能，用真实数据测试，再针对性优化
-- 5. 使用 EXPLAIN QUERY PLAN 分析查询性能
-- 6. 监控索引使用情况，及时清理无用索引

-- ============================================================
-- 未来优化建议
-- ============================================================

-- 如果发现特定查询慢，应该：
-- 1. 使用 EXPLAIN QUERY PLAN 分析查询执行计划
-- 2. 确认查询频率和数据规模
-- 3. 针对性添加索引并测试效果
-- 4. 监控索引使用情况 (SQLite 的 pragma index_info)
-- 5. 不要"猜测"需要什么索引，用数据说话

-- 常见查询模式索引建议：
-- - 按用户+收藏状态查询: idx_bookmarks_favorites (已存在)
-- - 按用户+归档状态查询: idx_bookmarks_active_users (已存在)
-- - 按用户+阅读状态查询: idx_bookmarks_unread (已存在)
-- - 标签统计查询: idx_tags_user_usage_created (已存在)
-- - 书签标签关联: idx_bookmark_tags_* (已存在)