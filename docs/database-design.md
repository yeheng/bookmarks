# 数据库设计文档

## 概述

本文档详细描述了书签应用的数据库设计，使用 SQLite 作为主数据库。设计遵循第三范式，确保数据一致性和完整性，同时考虑性能优化和扩展性。

## 数据库选择

### SQLite 3+

选择 SQLite 的理由：

- **轻量级部署**: 无需独立数据库服务器，适合中小型应用
- **零配置**: 开箱即用，减少部署复杂度
- **ACID 特性**: 保证数据一致性
- **跨平台**: 支持所有主流操作系统
- **高性能**: 对于读密集型应用性能优异
- **开发友好**: 本地开发环境简单，便于快速迭代

## 数据库架构

### 核心实体关系图

```
Users (用户)
├── Collections (收藏夹) [1:N]
├── Bookmarks (书签) [1:N]
└── Tags (标签) [1:N]
    └── Bookmark_Tags (书签标签关联) [N:M]
        └── Bookmarks (书签) [N:1]
```

## 表结构设计

### 1. 用户表 (users)

```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(6)))),
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    avatar_url TEXT,
    is_active INTEGER DEFAULT 1,
    email_verified INTEGER DEFAULT 0,
    email_verification_token TEXT,
    password_reset_token TEXT,
    password_reset_expires_at DATETIME,
    last_login_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_active ON users(is_active);
CREATE INDEX idx_users_email_verified ON users(email_verified);
```

**字段说明：**

- `id`: 主键，使用TEXT格式存储UUID
- `username`: 用户名，唯一约束
- `email`: 邮箱地址，唯一约束，用于登录
- `password_hash`: 加密后的密码，使用 bcrypt 存储
- `avatar_url`: 头像 URL（可选）
- `is_active`: 用户是否激活（INTEGER，1=激活，0=未激活）
- `email_verified`: 邮箱是否已验证（INTEGER，1=已验证，0=未验证）
- `email_verification_token`: 邮箱验证令牌
- `password_reset_token`: 密码重置令牌
- `password_reset_expires_at`: 密码重置令牌过期时间
- `last_login_at`: 最后登录时间
- `created_at`: 创建时间
- `updated_at`: 更新时间

### 2. 收藏夹表 (collections)

```sql
CREATE TABLE collections (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(6)))),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT DEFAULT '#3b82f6',
    icon TEXT DEFAULT 'folder',
    sort_order INTEGER DEFAULT 0,
    is_default INTEGER DEFAULT 0,
    is_public INTEGER DEFAULT 0,
    parent_id TEXT REFERENCES collections(id) ON DELETE CASCADE,
    bookmark_count INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT collections_user_name_unique UNIQUE(user_id, name)
);

-- 索引
CREATE INDEX idx_collections_user_id ON collections(user_id);
CREATE INDEX idx_collections_parent_id ON collections(parent_id);
CREATE INDEX idx_collections_sort_order ON collections(sort_order);
CREATE INDEX idx_collections_is_default ON collections(is_default);
CREATE INDEX idx_collections_is_public ON collections(is_public);
```

**字段说明：**

- `id`: 主键，使用TEXT格式存储UUID
- `user_id`: 用户 ID，外键关联用户表
- `name`: 收藏夹名称
- `description`: 收藏夹描述（可选）
- `color`: 收藏夹颜色，用于 UI 显示
- `icon`: 图标名称
- `sort_order`: 排序顺序
- `is_default`: 是否为默认收藏夹（INTEGER，1=是，0=否）
- `is_public`: 是否公开（INTEGER，1=公开，0=私有）
- `parent_id`: 父收藏夹 ID，支持嵌套收藏夹
- `bookmark_count`: 书签数量（冗余字段，用于性能优化）
- `created_at`: 创建时间
- `updated_at`: 更新时间

### 3. 书签表 (bookmarks)

```sql
CREATE TABLE bookmarks (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(6)))),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    collection_id TEXT REFERENCES collections(id) ON DELETE SET NULL,
    title TEXT NOT NULL,
    url TEXT NOT NULL,
    description TEXT,
    favicon_url TEXT,
    screenshot_url TEXT,
    thumbnail_url TEXT,
    is_favorite INTEGER DEFAULT 0,
    is_archived INTEGER DEFAULT 0,
    is_private INTEGER DEFAULT 0,
    is_read INTEGER DEFAULT 0,
    visit_count INTEGER DEFAULT 0,
    last_visited DATETIME,
    reading_time INTEGER,
    difficulty_level INTEGER CHECK (difficulty_level BETWEEN 1 AND 5),
    metadata TEXT DEFAULT '{}',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_bookmarks_user_id ON bookmarks(user_id);
CREATE INDEX idx_bookmarks_collection_id ON bookmarks(collection_id);
CREATE INDEX idx_bookmarks_is_favorite ON bookmarks(is_favorite);
CREATE INDEX idx_bookmarks_is_archived ON bookmarks(is_archived);
CREATE INDEX idx_bookmarks_is_private ON bookmarks(is_private);
CREATE INDEX idx_bookmarks_is_read ON bookmarks(is_read);
CREATE INDEX idx_bookmarks_created_at ON bookmarks(created_at DESC);
CREATE INDEX idx_bookmarks_last_visited ON bookmarks(last_visited DESC);
CREATE INDEX idx_bookmarks_visit_count ON bookmarks(visit_count DESC);

-- 全文搜索索引 (SQLite FTS5)
CREATE VIRTUAL TABLE bookmarks_fts USING fts5(
    title, 
    description,
    content='bookmarks',
    content_rowid='rowid'
);

-- FTS 触发器
CREATE TRIGGER bookmarks_fts_insert AFTER INSERT ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(rowid, title, description) 
    VALUES (new.rowid, new.title, new.description);
END;

CREATE TRIGGER bookmarks_fts_delete AFTER DELETE ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(bookmarks_fts, rowid, title, description) 
    VALUES ('delete', old.rowid, old.title, old.description);
END;

CREATE TRIGGER bookmarks_fts_update AFTER UPDATE ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(bookmarks_fts, rowid, title, description) 
    VALUES ('delete', old.rowid, old.title, old.description);
    INSERT INTO bookmarks_fts(rowid, title, description) 
    VALUES (new.rowid, new.title, new.description);
END;
```

**字段说明：**

- `id`: 主键，使用TEXT格式存储UUID
- `user_id`: 用户 ID
- `collection_id`: 收藏夹 ID（可选）
- `title`: 书签标题
- `url`: 书签 URL
- `description`: 书签描述（可选）
- `favicon_url`: 网站图标 URL
- `screenshot_url`: 网页截图 URL
- `thumbnail_url`: 缩略图 URL
- `is_favorite`: 是否收藏（INTEGER，1=收藏，0=未收藏）
- `is_archived`: 是否归档（INTEGER，1=归档，0=未归档）
- `is_private`: 是否私有（INTEGER，1=私有，0=公开）
- `is_read`: 是否已读（INTEGER，1=已读，0=未读）
- `visit_count`: 访问次数
- `last_visited`: 最后访问时间
- `reading_time`: 预估阅读时间（分钟）
- `difficulty_level`: 难度等级（1-5）
- `metadata`: JSON 格式的额外元数据（TEXT格式存储）
- `created_at`: 创建时间
- `updated_at`: 更新时间

### 4. 标签表 (tags)

```sql
CREATE TABLE tags (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(6)))),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    color TEXT DEFAULT '#64748b',
    description TEXT,
    usage_count INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT tags_user_name_unique UNIQUE(user_id, name)
);

-- 索引
CREATE INDEX idx_tags_user_id ON tags(user_id);
CREATE INDEX idx_tags_name ON tags(name);
CREATE INDEX idx_tags_usage_count ON tags(usage_count DESC);
CREATE INDEX idx_tags_created_at ON tags(created_at DESC);
```

**字段说明：**

- `id`: 主键，使用TEXT格式存储UUID
- `user_id`: 用户 ID
- `name`: 标签名称
- `color`: 标签颜色
- `description`: 标签描述（可选）
- `usage_count`: 使用次数（冗余字段）
- `created_at`: 创建时间
- `updated_at`: 更新时间

### 5. 书签标签关联表 (bookmark_tags)

```sql
CREATE TABLE bookmark_tags (
    bookmark_id TEXT NOT NULL REFERENCES bookmarks(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    PRIMARY KEY (bookmark_id, tag_id)
);

-- 索引
CREATE INDEX idx_bookmark_tags_bookmark_id ON bookmark_tags(bookmark_id);
CREATE INDEX idx_bookmark_tags_tag_id ON bookmark_tags(tag_id);
```

**字段说明：**

- `bookmark_id`: 书签 ID
- `tag_id`: 标签 ID
- `created_at`: 关联创建时间

### 6. 用户会话表 (user_sessions)

SQLite版本暂不包含独立的会话表，JWT令牌管理通过应用层实现。

### 7. 系统日志表 (audit_logs)

SQLite版本暂不包含审计日志表，审计功能可通过应用层日志实现。

## 视图设计

### 1. 书签详情视图

```sql
CREATE VIEW bookmark_details AS
SELECT 
    b.*,
    c.name as collection_name,
    c.color as collection_color,
    u.username as owner_username
FROM bookmarks b
LEFT JOIN collections c ON b.collection_id = c.id
LEFT JOIN users u ON b.user_id = u.id;
```

注意：SQLite不支持array_agg函数，标签聚合需要通过应用层实现。

### 2. 用户统计视图

```sql
CREATE VIEW user_statistics AS
SELECT 
    u.id,
    u.username,
    u.email,
    COUNT(DISTINCT b.id) as total_bookmarks,
    COUNT(DISTINCT c.id) as total_collections,
    COUNT(DISTINCT t.id) as total_tags,
    COUNT(DISTINCT CASE WHEN b.is_favorite = 1 THEN b.id END) as favorite_bookmarks,
    COUNT(DISTINCT CASE WHEN b.is_archived = 1 THEN b.id END) as archived_bookmarks,
    COALESCE(SUM(b.visit_count), 0) as total_visits,
    MAX(b.last_visited) as last_bookmark_visit,
    u.created_at as user_created_at
FROM users u
LEFT JOIN bookmarks b ON u.id = b.user_id
LEFT JOIN collections c ON u.id = c.user_id
LEFT JOIN tags t ON u.id = t.user_id
GROUP BY u.id, u.username, u.email, u.created_at;
```

### 3. 标签云视图

```sql
CREATE VIEW tag_cloud AS
SELECT 
    t.id,
    t.name,
    t.color,
    t.usage_count,
    COUNT(bt.bookmark_id) as actual_bookmark_count,
    COUNT(CASE WHEN b.is_favorite = 1 THEN 1 END) as favorite_bookmark_count,
    MAX(b.created_at) as last_used_at
FROM tags t
LEFT JOIN bookmark_tags bt ON t.id = bt.tag_id
LEFT JOIN bookmarks b ON bt.bookmark_id = b.id
GROUP BY t.id, t.name, t.color, t.usage_count;
```

## 触发器和函数

### 1. 更新时间戳触发器

SQLite中每个表的更新时间戳触发器已在表创建时定义：

```sql
-- 用户表更新时间戳触发器
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW BEGIN
        UPDATE users SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

-- 收藏夹表更新时间戳触发器
CREATE TRIGGER update_collections_updated_at BEFORE UPDATE ON collections
    FOR EACH ROW BEGIN
        UPDATE collections SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

-- 书签表更新时间戳触发器
CREATE TRIGGER update_bookmarks_updated_at BEFORE UPDATE ON bookmarks
    FOR EACH ROW BEGIN
        UPDATE bookmarks SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;

-- 标签表更新时间戳触发器
CREATE TRIGGER update_tags_updated_at BEFORE UPDATE ON tags
    FOR EACH ROW BEGIN
        UPDATE tags SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
    END;
```

### 2. 收藏夹书签计数触发器

```sql
-- 插入书签时更新收藏夹计数
CREATE TRIGGER update_collection_bookmark_count_insert
    AFTER INSERT ON bookmarks
    FOR EACH ROW BEGIN
        UPDATE collections 
        SET bookmark_count = bookmark_count + 1 
        WHERE id = NEW.collection_id;
    END;

-- 更新书签时更新收藏夹计数
CREATE TRIGGER update_collection_bookmark_count_update
    AFTER UPDATE ON bookmarks
    FOR EACH ROW BEGIN
        IF OLD.collection_id IS NOT NEW.collection_id THEN
            IF OLD.collection_id IS NOT NULL THEN
                UPDATE collections 
                SET bookmark_count = bookmark_count - 1 
                WHERE id = OLD.collection_id;
            END IF;
            IF NEW.collection_id IS NOT NULL THEN
                UPDATE collections 
                SET bookmark_count = bookmark_count + 1 
                WHERE id = NEW.collection_id;
            END IF;
        END IF;
    END;

-- 删除书签时更新收藏夹计数
CREATE TRIGGER update_collection_bookmark_count_delete
    AFTER DELETE ON bookmarks
    FOR EACH ROW BEGIN
        IF OLD.collection_id IS NOT NULL THEN
            UPDATE collections 
            SET bookmark_count = bookmark_count - 1 
            WHERE id = OLD.collection_id;
        END IF;
    END;
```

### 3. 标签使用计数触发器

```sql
-- 插入书签标签关联时更新标签使用计数
CREATE TRIGGER update_tag_usage_count_insert
    AFTER INSERT ON bookmark_tags
    FOR EACH ROW BEGIN
        UPDATE tags 
        SET usage_count = usage_count + 1 
        WHERE id = NEW.tag_id;
    END;

-- 删除书签标签关联时更新标签使用计数
CREATE TRIGGER update_tag_usage_count_delete
    AFTER DELETE ON bookmark_tags
    FOR EACH ROW BEGIN
        UPDATE tags 
        SET usage_count = usage_count - 1 
        WHERE id = OLD.tag_id;
    END;
```

### 4. 审计日志触发器

SQLite版本暂不包含审计日志功能，如需审计功能可通过应用层实现。

## 数据库函数

SQLite不支持存储过程和自定义函数，复杂的业务逻辑建议在应用层实现。以下是一些常用的查询模式：

### 1. 全文搜索查询

```sql
-- 使用FTS5进行全文搜索
SELECT b.*, c.name as collection_name
FROM bookmarks b
LEFT JOIN collections c ON b.collection_id = c.id
JOIN bookmarks_fts fts ON b.rowid = fts.rowid
WHERE b.user_id = ? 
  AND bookmarks_fts MATCH ?
ORDER BY rank
LIMIT ? OFFSET ?;
```

### 2. 统计分析查询

```sql
-- 用户书签统计
SELECT 
    COUNT(*) as total_bookmarks,
    COUNT(CASE WHEN is_favorite = 1 THEN 1 END) as favorite_count,
    COUNT(CASE WHEN is_archived = 1 THEN 1 END) as archived_count,
    COUNT(CASE WHEN is_read = 0 THEN 1 END) as unread_count,
    SUM(visit_count) as total_visits
FROM bookmarks 
WHERE user_id = ?;

-- 按日期统计书签添加
SELECT 
    DATE(created_at) as date,
    COUNT(*) as bookmarks_added
FROM bookmarks 
WHERE user_id = ? 
  AND DATE(created_at) >= DATE('now', '-30 days')
GROUP BY DATE(created_at)
ORDER BY date DESC;
```

## 数据迁移策略

### 1. 版本控制

使用 SQLx migrate 进行数据库版本控制：

```bash
# 创建新迁移
sqlx migrate add create_users_table

# 应用迁移
sqlx migrate run

# 回滚迁移（SQLite有限支持）
sqlx migrate revert
```

### 2. 迁移文件示例

```sql
-- migrations/20231201000001_create_users_table.sql
CREATE TABLE users (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(6)))),
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

```sql
-- migrations/20231201000002_create_collections_table.sql
CREATE TABLE collections (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(4))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(2))) || '-' || lower(hex(randomblob(6)))),
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    color TEXT DEFAULT '#3b82f6',
    sort_order INTEGER DEFAULT 0,
    is_default INTEGER DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    
    CONSTRAINT collections_user_name_unique UNIQUE(user_id, name)
);
```

### 3. 数据迁移脚本

```sql
-- 数据迁移：从旧版本升级到新版本
-- migrations/20231201000010_migrate_bookmark_metadata.sql

-- 创建新的 metadata 列
ALTER TABLE bookmarks ADD COLUMN metadata TEXT DEFAULT '{}';

-- 迁移现有的 difficulty_level 和 reading_time 到 metadata
UPDATE bookmarks 
SET metadata = json_object(
    'difficulty_level', difficulty_level,
    'reading_time', reading_time
)
WHERE difficulty_level IS NOT NULL OR reading_time IS NOT NULL;

-- 删除旧的列（如果不再需要）
-- ALTER TABLE bookmarks DROP COLUMN difficulty_level;
-- ALTER TABLE bookmarks DROP COLUMN reading_time;
```

## 性能优化

### 1. 索引策略

```sql
-- 复合索引优化查询
CREATE INDEX idx_bookmarks_user_collection_created ON bookmarks(user_id, collection_id, created_at DESC);
CREATE INDEX idx_bookmarks_user_favorite_created ON bookmarks(user_id, is_favorite, created_at DESC);
CREATE INDEX idx_bookmarks_user_archived_created ON bookmarks(user_id, is_archived, created_at DESC);

-- 部分索引（SQLite不支持部分索引，使用标准索引替代）
CREATE INDEX idx_bookmarks_unread ON bookmarks(user_id, is_read, created_at DESC);
CREATE INDEX idx_bookmarks_recently_visited ON bookmarks(user_id, last_visited DESC);

-- JSON查询优化（SQLite使用JSON1扩展）
CREATE INDEX idx_bookmarks_metadata ON bookmarks(metadata);
```

### 2. 分区策略

SQLite不支持原生表分区，如需大数据量处理可考虑：
- 按时间分表（bookmarks_2023_12, bookmarks_2024_01等）
- 使用UNION ALL视图进行查询
- 定期归档旧数据

### 3. 查询优化

```sql
-- 使用子查询优化复杂查询
SELECT 
    b.*,
    c.name as collection_name
FROM bookmarks b
LEFT JOIN collections c ON b.collection_id = c.id
WHERE b.user_id = ? 
  AND b.is_archived = 0
ORDER BY b.created_at DESC
LIMIT ? OFFSET ?;

-- 标签聚合查询
SELECT 
    b.*,
    GROUP_CONCAT(t.name, ',') as tags
FROM bookmarks b
LEFT JOIN bookmark_tags bt ON b.id = bt.bookmark_id
LEFT JOIN tags t ON bt.tag_id = t.id
WHERE b.user_id = ?
GROUP BY b.id
ORDER BY b.created_at DESC;
```

## 备份和恢复

### 1. 备份策略

```bash
# SQLite数据库备份
sqlite3 bookmarks.db ".backup backup_$(date +%Y%m%d_%H%M%S).db"

# 压缩备份
sqlite3 bookmarks.db ".backup backup_temp.db" && gzip backup_temp.db && mv backup_temp.db.gz backup_$(date +%Y%m%d_%H%M%S).db.gz

# 导出SQL格式
sqlite3 bookmarks_db ".dump" > backup_$(date +%Y%m%d_%H%M%S).sql

# 仅数据导出
sqlite3 bookmarks_db ".dump --data-only" > data_backup_$(date +%Y%m%d_%H%M%S).sql

# 仅结构导出
sqlite3 bookmarks_db ".dump --schema-only" > schema_backup_$(date +%Y%m%d_%H%M%S).sql
```

### 2. 恢复策略

```bash
# 从备份文件恢复
cp backup_20231201_120000.db bookmarks.db

# 从SQL备份恢复
sqlite3 bookmarks.db < backup_20231201_120000.sql

# 从压缩备份恢复
gunzip -c backup_20231201_120000.db.gz > bookmarks.db
```

### 3. 自动化备份脚本

```bash
#!/bin/bash
# backup_database.sh

DB_FILE="bookmarks.db"
BACKUP_DIR="/var/backups/sqlite"
DATE=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=30

# 创建备份目录
mkdir -p $BACKUP_DIR

# 执行备份
sqlite3 $DB_FILE ".backup $BACKUP_DIR/backup_$DATE.db"

# 压缩备份
gzip $BACKUP_DIR/backup_$DATE.db

# 删除旧备份
find $BACKUP_DIR -name "backup_*.db.gz" -mtime +$RETENTION_DAYS -delete

echo "Backup completed: backup_$DATE.db.gz"
```

## 安全配置

### 1. 数据库文件安全

SQLite是文件数据库，安全主要通过文件系统权限控制：

```bash
# 设置数据库文件权限
chmod 600 bookmarks.db
chmod 700 /path/to/database/directory

# 设置文件所有者
chown app_user:app_group bookmarks.db

# 启用WAL模式时的额外文件
chmod 600 bookmarks.db-wal
chmod 600 bookmarks.db-shm
```

### 2. 数据库加密

考虑使用SQLite扩展或商业版进行加密：

```sql
-- 启用加密（需要SQLCipher）
PRAGMA key = 'your-encryption-key';

-- 验证数据库完整性
PRAGMA cipher_integrity_check;
```

### 3. 应用层数据访问控制

由于SQLite不支持行级安全，数据隔离需要在应用层实现：

```rust
// 示例：Rust应用层权限检查
async fn get_user_bookmarks(user_id: &str, pool: &SqlitePool) -> Result<Vec<Bookmark>> {
    sqlx::query_as!(
        Bookmark,
        "SELECT * FROM bookmarks WHERE user_id = ?",
        user_id
    )
    .fetch_all(pool)
    .await
}
```

## 监控和维护

### 1. 性能监控查询

```sql
-- 查看表统计信息
SELECT 
    name,
    sql,
    tbl_name,
    rootpage,
    sql
FROM sqlite_master 
WHERE type = 'table'
ORDER BY name;

-- 查看索引信息
SELECT 
    name,
    tbl_name,
    sql
FROM sqlite_master 
WHERE type = 'index'
ORDER BY tbl_name, name;

-- 分析查询计划
EXPLAIN QUERY PLAN 
SELECT * FROM bookmarks 
WHERE user_id = ? AND is_favorite = 1 
ORDER BY created_at DESC 
LIMIT 10;
```

### 2. 维护脚本

```sql
-- 分析表统计信息
ANALYZE;

-- 清理数据库
VACUUM;

-- 检查数据库完整性
PRAGMA integrity_check;

-- 优化数据库文件大小
PRAGMA optimize;

-- 查看数据库设置
PRAGMA compile_options;
```

### 3. 性能优化设置

```sql
-- 启用WAL模式（提高并发性能）
PRAGMA journal_mode = WAL;

-- 设置同步模式（平衡性能和安全性）
PRAGMA synchronous = NORMAL;

-- 设置缓存大小（根据内存调整）
PRAGMA cache_size = 10000;

-- 启用外键约束
PRAGMA foreign_keys = ON;

-- 设置临时存储
PRAGMA temp_store = MEMORY;
```

---

这个数据库设计基于SQLite提供了完整、轻量级和高性能的数据存储解决方案，支持书签应用的所有核心功能，并为未来的功能扩展预留了空间。SQLite的选择使得部署和维护更加简单，适合中小型应用和快速开发迭代。
