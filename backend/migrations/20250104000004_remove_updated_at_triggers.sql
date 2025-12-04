-- 移除所有 updated_at 触发器
--
-- 原因:
-- 1. 当前触发器设计有误: 在 BEFORE UPDATE 中执行 UPDATE 语句,逻辑错误
-- 2. updated_at 应该由应用层 ORM 控制,而不是数据库触发器
-- 3. 应用层控制时间戳更可靠,更容易测试和调试
-- 4. 减少数据库端的复杂性,降低维护成本
--
-- 应用层实现:
-- 在 Rust 的 BookmarkService/CollectionService 等服务中,
-- 所有 UPDATE 操作都已经显式设置:
--   updated_at = CAST(strftime('%s', 'now') AS INTEGER)
--
-- Linus 说: "Theory and practice sometimes clash. Theory loses."
-- 触发器理论上很美好,实践中却增加了复杂性和出错风险

-- 删除所有 updated_at 触发器
DROP TRIGGER IF EXISTS update_users_updated_at;
DROP TRIGGER IF EXISTS update_collections_updated_at;
DROP TRIGGER IF EXISTS update_bookmarks_updated_at;
DROP TRIGGER IF EXISTS update_tags_updated_at;
DROP TRIGGER IF EXISTS update_user_settings_updated_at;
