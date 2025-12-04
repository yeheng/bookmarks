-- 移除 bookmark_stats 表和相关触发器
--
-- 原因:
-- 1. bookmark_stats 表从未在应用层被引用,纯粹的过度设计
-- 2. 触发器增加了写入成本 (每次 INSERT/UPDATE/DELETE 都要额外更新统计)
-- 3. 统计数据可以在需要时通过简单的 COUNT() 实时计算
-- 4. 对于书签管理系统的数据规模,实时计算的性能完全可接受
--
-- Linus 说: "The best code is no code."
-- 删掉不需要的复杂性,让系统更简单、更可靠

-- 删除触发器 (如果存在)
DROP TRIGGER IF EXISTS update_bookmark_stats_after_insert;
DROP TRIGGER IF EXISTS update_bookmark_stats_after_delete;
DROP TRIGGER IF EXISTS update_bookmark_stats_after_update;

-- 删除 bookmark_stats 表 (如果存在)
DROP TABLE IF EXISTS bookmark_stats;
