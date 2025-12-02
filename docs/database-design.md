# 数据库设计文档

## 概述

本文档详细描述了书签应用的数据库设计，使用 PostgreSQL 作为主数据库。设计遵循第三范式，确保数据一致性和完整性，同时考虑性能优化和扩展性。

## 数据库选择

### PostgreSQL 15+

选择 PostgreSQL 的理由：
- **强大的 JSON 支持**: 适合存储灵活的书签元数据
- **全文搜索功能**: 内置的全文搜索支持
- **ACID 特性**: 保证数据一致性
- **UUID 支持**: 原生支持 UUID 数据类型
- **索引优化**: 支持多种索引类型，包括 GIN 索引
- **扩展性**: 支持水平和垂直扩展

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
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    avatar_url VARCHAR(2048),
    is_active BOOLEAN DEFAULT TRUE,
    email_verified BOOLEAN DEFAULT FALSE,
    email_verification_token VARCHAR(255),
    password_reset_token VARCHAR(255),
    password_reset_expires_at TIMESTAMP WITH TIME ZONE,
    last_login_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_active ON users(is_active);
CREATE INDEX idx_users_email_verified ON users(email_verified);
```

**字段说明：**
- `id`: 主键，使用 UUID 确保全局唯一性
- `username`: 用户名，唯一约束
- `email`: 邮箱地址，唯一约束，用于登录
- `password_hash`: 加密后的密码，使用 bcrypt 存储
- `avatar_url`: 头像 URL（可选）
- `is_active`: 用户是否激活
- `email_verified`: 邮箱是否已验证
- `email_verification_token`: 邮箱验证令牌
- `password_reset_token`: 密码重置令牌
- `password_reset_expires_at`: 密码重置令牌过期时间
- `last_login_at`: 最后登录时间
- `created_at`: 创建时间
- `updated_at`: 更新时间

### 2. 收藏夹表 (collections)

```sql
CREATE TABLE collections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    color VARCHAR(7) DEFAULT '#3b82f6', -- HEX 颜色代码
    icon VARCHAR(50) DEFAULT 'folder',
    sort_order INTEGER DEFAULT 0,
    is_default BOOLEAN DEFAULT FALSE,
    is_public BOOLEAN DEFAULT FALSE,
    parent_id UUID REFERENCES collections(id) ON DELETE CASCADE,
    bookmark_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
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
- `id`: 主键
- `user_id`: 用户 ID，外键关联用户表
- `name`: 收藏夹名称
- `description`: 收藏夹描述（可选）
- `color`: 收藏夹颜色，用于 UI 显示
- `icon`: 图标名称
- `sort_order`: 排序顺序
- `is_default`: 是否为默认收藏夹
- `is_public`: 是否公开（未来扩展功能）
- `parent_id`: 父收藏夹 ID，支持嵌套收藏夹
- `bookmark_count`: 书签数量（冗余字段，用于性能优化）
- `created_at`: 创建时间
- `updated_at`: 更新时间

### 3. 书签表 (bookmarks)

```sql
CREATE TABLE bookmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    collection_id UUID REFERENCES collections(id) ON DELETE SET NULL,
    title VARCHAR(255) NOT NULL,
    url VARCHAR(2048) NOT NULL,
    description TEXT,
    favicon_url VARCHAR(2048),
    screenshot_url VARCHAR(2048),
    thumbnail_url VARCHAR(2048),
    is_favorite BOOLEAN DEFAULT FALSE,
    is_archived BOOLEAN DEFAULT FALSE,
    is_private BOOLEAN DEFAULT FALSE,
    is_read BOOLEAN DEFAULT FALSE,
    visit_count INTEGER DEFAULT 0,
    last_visited TIMESTAMP WITH TIME ZONE,
    reading_time INTEGER, -- 预估阅读时间（分钟）
    difficulty_level INTEGER CHECK (difficulty_level BETWEEN 1 AND 5),
    metadata JSONB DEFAULT '{}', -- 额外的元数据
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
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

-- 全文搜索索引
CREATE INDEX bookmarks_search_idx ON bookmarks USING GIN (
    to_tsvector('english', title || ' ' || COALESCE(description, ''))
);

-- URL 域名索引（用于域名统计）
CREATE INDEX bookmarks_domain_idx ON bookmarks (regexp_replace(url, '^https?://([^/]+).*', '\1'));
```

**字段说明：**
- `id`: 主键
- `user_id`: 用户 ID
- `collection_id`: 收藏夹 ID（可选）
- `title`: 书签标题
- `url`: 书签 URL
- `description`: 书签描述（可选）
- `favicon_url`: 网站图标 URL
- `screenshot_url`: 网页截图 URL
- `thumbnail_url`: 缩略图 URL
- `is_favorite`: 是否收藏
- `is_archived`: 是否归档
- `is_private`: 是否私有
- `is_read`: 是否已读
- `visit_count`: 访问次数
- `last_visited`: 最后访问时间
- `reading_time`: 预估阅读时间（分钟）
- `difficulty_level`: 难度等级（1-5）
- `metadata`: JSON 格式的额外元数据
- `created_at`: 创建时间
- `updated_at`: 更新时间

### 4. 标签表 (tags)

```sql
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(50) NOT NULL,
    color VARCHAR(7) DEFAULT '#64748b',
    description TEXT,
    usage_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    CONSTRAINT tags_user_name_unique UNIQUE(user_id, name)
);

-- 索引
CREATE INDEX idx_tags_user_id ON tags(user_id);
CREATE INDEX idx_tags_name ON tags(name);
CREATE INDEX idx_tags_usage_count ON tags(usage_count DESC);
CREATE INDEX idx_tags_created_at ON tags(created_at DESC);
```

**字段说明：**
- `id`: 主键
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
    bookmark_id UUID NOT NULL REFERENCES bookmarks(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
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

```sql
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token VARCHAR(255) UNIQUE NOT NULL,
    refresh_token VARCHAR(255) UNIQUE NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    ip_address INET,
    user_agent TEXT,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_session_token ON user_sessions(session_token);
CREATE INDEX idx_user_sessions_refresh_token ON user_sessions(refresh_token);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX idx_user_sessions_is_active ON user_sessions(is_active);
```

### 7. 系统日志表 (audit_logs)

```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id UUID,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_resource_type ON audit_logs(resource_type);
CREATE INDEX idx_audit_logs_resource_id ON audit_logs(resource_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
```

## 视图设计

### 1. 书签详情视图

```sql
CREATE VIEW bookmark_details AS
SELECT 
    b.*,
    c.name as collection_name,
    c.color as collection_color,
    COALESCE(
        array_agg(t.name) FILTER (WHERE t.name IS NOT NULL),
        ARRAY[]::VARCHAR[]
    ) as tags,
    u.username as owner_username
FROM bookmarks b
LEFT JOIN collections c ON b.collection_id = c.id
LEFT JOIN bookmark_tags bt ON b.id = bt.bookmark_id
LEFT JOIN tags t ON bt.tag_id = t.id
LEFT JOIN users u ON b.user_id = u.id
GROUP BY b.id, c.name, c.color, u.username;
```

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
    COUNT(DISTINCT CASE WHEN b.is_favorite = TRUE THEN b.id END) as favorite_bookmarks,
    COUNT(DISTINCT CASE WHEN b.is_archived = TRUE THEN b.id END) as archived_bookmarks,
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
    COUNT(CASE WHEN b.is_favorite = TRUE THEN 1 END) as favorite_bookmark_count,
    MAX(b.created_at) as last_used_at
FROM tags t
LEFT JOIN bookmark_tags bt ON t.id = bt.tag_id
LEFT JOIN bookmarks b ON bt.bookmark_id = b.id
GROUP BY t.id, t.name, t.color, t.usage_count;
```

## 触发器和函数

### 1. 更新时间戳触发器

```sql
-- 创建更新时间戳函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 为需要的表添加触发器
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_collections_updated_at BEFORE UPDATE ON collections
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_bookmarks_updated_at BEFORE UPDATE ON bookmarks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tags_updated_at BEFORE UPDATE ON tags
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

### 2. 收藏夹书签计数触发器

```sql
-- 更新收藏夹书签计数函数
CREATE OR REPLACE FUNCTION update_collection_bookmark_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE collections 
        SET bookmark_count = bookmark_count + 1 
        WHERE id = NEW.collection_id;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.collection_id IS DISTINCT FROM NEW.collection_id THEN
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
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        IF OLD.collection_id IS NOT NULL THEN
            UPDATE collections 
            SET bookmark_count = bookmark_count - 1 
            WHERE id = OLD.collection_id;
        END IF;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器
CREATE TRIGGER update_collection_bookmark_count_trigger
    AFTER INSERT OR UPDATE OR DELETE ON bookmarks
    FOR EACH ROW EXECUTE FUNCTION update_collection_bookmark_count();
```

### 3. 标签使用计数触发器

```sql
-- 更新标签使用计数函数
CREATE OR REPLACE FUNCTION update_tag_usage_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE tags 
        SET usage_count = usage_count + 1 
        WHERE id = NEW.tag_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE tags 
        SET usage_count = usage_count - 1 
        WHERE id = OLD.tag_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器
CREATE TRIGGER update_tag_usage_count_trigger
    AFTER INSERT OR DELETE ON bookmark_tags
    FOR EACH ROW EXECUTE FUNCTION update_tag_usage_count();
```

### 4. 审计日志触发器

```sql
-- 创建审计日志函数
CREATE OR REPLACE FUNCTION audit_trigger_function()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        INSERT INTO audit_logs (user_id, action, resource_type, resource_id, old_values)
        VALUES (
            COALESCE(OLD.user_id, current_setting('app.current_user_id', true)::UUID),
            TG_OP,
            TG_TABLE_NAME,
            OLD.id,
            row_to_json(OLD)
        );
        RETURN OLD;
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO audit_logs (user_id, action, resource_type, resource_id, old_values, new_values)
        VALUES (
            COALESCE(NEW.user_id, OLD.user_id, current_setting('app.current_user_id', true)::UUID),
            TG_OP,
            TG_TABLE_NAME,
            COALESCE(NEW.id, OLD.id),
            row_to_json(OLD),
            row_to_json(NEW)
        );
        RETURN NEW;
    ELSIF TG_OP = 'INSERT' THEN
        INSERT INTO audit_logs (user_id, action, resource_type, resource_id, new_values)
        VALUES (
            COALESCE(NEW.user_id, current_setting('app.current_user_id', true)::UUID),
            TG_OP,
            TG_TABLE_NAME,
            NEW.id,
            row_to_json(NEW)
        );
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- 为重要表添加审计触发器
CREATE TRIGGER audit_bookmarks_trigger
    AFTER INSERT OR UPDATE OR DELETE ON bookmarks
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_collections_trigger
    AFTER INSERT OR UPDATE OR DELETE ON collections
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();
```

## 数据库函数

### 1. 全文搜索函数

```sql
-- 高级书签搜索函数
CREATE OR REPLACE FUNCTION search_bookmarks(
    p_user_id UUID,
    p_query TEXT DEFAULT NULL,
    p_collection_id UUID DEFAULT NULL,
    p_tags TEXT[] DEFAULT NULL,
    p_is_favorite BOOLEAN DEFAULT NULL,
    p_is_archived BOOLEAN DEFAULT NULL,
    p_limit INTEGER DEFAULT 50,
    p_offset INTEGER DEFAULT 0
)
RETURNS TABLE (
    id UUID,
    title VARCHAR,
    url VARCHAR,
    description TEXT,
    is_favorite BOOLEAN,
    is_archived BOOLEAN,
    visit_count INTEGER,
    created_at TIMESTAMP WITH TIME ZONE,
    tags TEXT[],
    collection_name VARCHAR,
    rank REAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        b.id,
        b.title,
        b.url,
        b.description,
        b.is_favorite,
        b.is_archived,
        b.visit_count,
        b.created_at,
        COALESCE(tag_array, ARRAY[]::TEXT[]) as tags,
        c.name as collection_name,
        CASE 
            WHEN p_query IS NOT NULL THEN 
                ts_rank(
                    to_tsvector('english', b.title || ' ' || COALESCE(b.description, '')),
                    plainto_tsquery('english', p_query)
                )
            ELSE 0
        END as rank
    FROM bookmarks b
    LEFT JOIN collections c ON b.collection_id = c.id
    LEFT JOIN (
        SELECT 
            bt.bookmark_id,
            array_agg(t.name) as tag_array
        FROM bookmark_tags bt
        JOIN tags t ON bt.tag_id = t.id
        GROUP BY bt.bookmark_id
    ) tag_data ON b.id = tag_data.bookmark_id
    WHERE 
        b.user_id = p_user_id
        AND (p_query IS NULL OR to_tsvector('english', b.title || ' ' || COALESCE(b.description, '')) @@ plainto_tsquery('english', p_query))
        AND (p_collection_id IS NULL OR b.collection_id = p_collection_id)
        AND (p_is_favorite IS NULL OR b.is_favorite = p_is_favorite)
        AND (p_is_archived IS NULL OR b.is_archived = p_is_archived)
        AND (p_tags IS NULL OR tag_data.tag_array @> p_tags)
    ORDER BY 
        CASE WHEN p_query IS NOT NULL THEN rank END DESC,
        b.created_at DESC
    LIMIT p_limit OFFSET p_offset;
END;
$$ LANGUAGE plpgsql;
```

### 2. 统计分析函数

```sql
-- 用户活动统计函数
CREATE OR REPLACE FUNCTION get_user_activity_stats(
    p_user_id UUID,
    p_start_date DATE DEFAULT NULL,
    p_end_date DATE DEFAULT NULL
)
RETURNS TABLE (
    date DATE,
    bookmarks_added INTEGER,
    bookmarks_visited INTEGER,
    tags_created INTEGER,
    collections_created INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        DATE(d.date) as date,
        COALESCE(bookmark_stats.added, 0) as bookmarks_added,
        COALESCE(visit_stats.visited, 0) as bookmarks_visited,
        COALESCE(tag_stats.created, 0) as tags_created,
        COALESCE(collection_stats.created, 0) as collections_created
    FROM generate_series(
        COALESCE(p_start_date, CURRENT_DATE - INTERVAL '30 days'),
        COALESCE(p_end_date, CURRENT_DATE),
        INTERVAL '1 day'
    ) d(date)
    LEFT JOIN (
        SELECT DATE(created_at) as date, COUNT(*) as added
        FROM bookmarks 
        WHERE user_id = p_user_id
        AND created_at >= COALESCE(p_start_date, CURRENT_DATE - INTERVAL '30 days')
        AND created_at <= COALESCE(p_end_date, CURRENT_DATE) + INTERVAL '1 day'
        GROUP BY DATE(created_at)
    ) bookmark_stats ON DATE(d.date) = bookmark_stats.date
    LEFT JOIN (
        SELECT DATE(last_visited) as date, COUNT(*) as visited
        FROM bookmarks 
        WHERE user_id = p_user_id
        AND last_visited >= COALESCE(p_start_date, CURRENT_DATE - INTERVAL '30 days')
        AND last_visited <= COALESCE(p_end_date, CURRENT_DATE) + INTERVAL '1 day'
        GROUP BY DATE(last_visited)
    ) visit_stats ON DATE(d.date) = visit_stats.date
    LEFT JOIN (
        SELECT DATE(created_at) as date, COUNT(*) as created
        FROM tags 
        WHERE user_id = p_user_id
        AND created_at >= COALESCE(p_start_date, CURRENT_DATE - INTERVAL '30 days')
        AND created_at <= COALESCE(p_end_date, CURRENT_DATE) + INTERVAL '1 day'
        GROUP BY DATE(created_at)
    ) tag_stats ON DATE(d.date) = tag_stats.date
    LEFT JOIN (
        SELECT DATE(created_at) as date, COUNT(*) as created
        FROM collections 
        WHERE user_id = p_user_id
        AND created_at >= COALESCE(p_start_date, CURRENT_DATE - INTERVAL '30 days')
        AND created_at <= COALESCE(p_end_date, CURRENT_DATE) + INTERVAL '1 day'
        GROUP BY DATE(created_at)
    ) collection_stats ON DATE(d.date) = collection_stats.date
    ORDER BY d.date;
END;
$$ LANGUAGE plpgsql;
```

## 数据迁移策略

### 1. 版本控制

使用 SQLx migrate 进行数据库版本控制：

```bash
# 创建新迁移
sqlx migrate add create_users_table

# 应用迁移
sqlx migrate run

# 回滚迁移
sqlx migrate revert
```

### 2. 迁移文件示例

```sql
-- migrations/20231201000001_create_users_table.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

```sql
-- migrations/20231201000002_create_collections_table.sql
CREATE TABLE collections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    color VARCHAR(7) DEFAULT '#3b82f6',
    sort_order INTEGER DEFAULT 0,
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    CONSTRAINT collections_user_name_unique UNIQUE(user_id, name)
);
```

### 3. 数据迁移脚本

```sql
-- 数据迁移：从旧版本升级到新版本
-- migrations/20231201000010_migrate_bookmark_metadata.sql

-- 创建新的 metadata 列
ALTER TABLE bookmarks ADD COLUMN IF NOT EXISTS metadata JSONB DEFAULT '{}';

-- 迁移现有的 difficulty_level 和 reading_time 到 metadata
UPDATE bookmarks 
SET metadata = jsonb_set(
    jsonb_set(
        metadata,
        '{difficulty_level}',
        to_jsonb(difficulty_level)
    ),
    '{reading_time}',
    to_jsonb(reading_time)
)
WHERE difficulty_level IS NOT NULL OR reading_time IS NOT NULL;

-- 删除旧的列（如果不再需要）
-- ALTER TABLE bookmarks DROP COLUMN IF EXISTS difficulty_level;
-- ALTER TABLE bookmarks DROP COLUMN IF EXISTS reading_time;
```

## 性能优化

### 1. 索引策略

```sql
-- 复合索引优化查询
CREATE INDEX idx_bookmarks_user_collection_created ON bookmarks(user_id, collection_id, created_at DESC);
CREATE INDEX idx_bookmarks_user_favorite_created ON bookmarks(user_id, is_favorite, created_at DESC) WHERE is_favorite = TRUE;
CREATE INDEX idx_bookmarks_user_archived_created ON bookmarks(user_id, is_archived, created_at DESC) WHERE is_archived = TRUE;

-- 部分索引（针对特定查询模式）
CREATE INDEX idx_bookmarks_unread ON bookmarks(user_id, is_read, created_at DESC) WHERE is_read = FALSE;
CREATE INDEX idx_bookmarks_recently_visited ON bookmarks(user_id, last_visited DESC) WHERE last_visited > NOW() - INTERVAL '7 days';

-- GIN 索引用于 JSONB 查询
CREATE INDEX idx_bookmarks_metadata_gin ON bookmarks USING GIN (metadata);
```

### 2. 分区策略

```sql
-- 按时间分区审计日志表（PostgreSQL 10+）
CREATE TABLE audit_logs_partitioned (
    LIKE audit_logs INCLUDING ALL
) PARTITION BY RANGE (created_at);

-- 创建月度分区
CREATE TABLE audit_logs_2023_12 PARTITION OF audit_logs_partitioned
    FOR VALUES FROM ('2023-12-01') TO ('2024-01-01');

CREATE TABLE audit_logs_2024_01 PARTITION OF audit_logs_partitioned
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
```

### 3. 查询优化

```sql
-- 使用 CTE 优化复杂查询
WITH user_bookmarks AS (
    SELECT * FROM bookmarks 
    WHERE user_id = $1 AND is_archived = FALSE
),
bookmark_tags AS (
    SELECT 
        b.id,
        array_agg(t.name) as tags
    FROM user_bookmarks b
    LEFT JOIN bookmark_tags bt ON b.id = bt.bookmark_id
    LEFT JOIN tags t ON bt.tag_id = t.id
    GROUP BY b.id
)
SELECT 
    b.*,
    bt.tags
FROM user_bookmarks b
LEFT JOIN bookmark_tags bt ON b.id = bt.id
ORDER BY b.created_at DESC
LIMIT $2 OFFSET $3;
```

## 备份和恢复

### 1. 备份策略

```bash
# 全量备份
pg_dump -h localhost -U bookmarks_user -d bookmarks_db > backup_$(date +%Y%m%d_%H%M%S).sql

# 压缩备份
pg_dump -h localhost -U bookmarks_user -d bookmarks_db | gzip > backup_$(date +%Y%m%d_%H%M%S).sql.gz

# 仅数据备份
pg_dump -h localhost -U bookmarks_user -d bookmarks_db --data-only > data_backup_$(date +%Y%m%d_%H%M%S).sql

# 仅结构备份
pg_dump -h localhost -U bookmarks_user -d bookmarks_db --schema-only > schema_backup_$(date +%Y%m%d_%H%M%S).sql
```

### 2. 恢复策略

```bash
# 从备份恢复
psql -h localhost -U bookmarks_user -d bookmarks_db < backup_20231201_120000.sql

# 从压缩备份恢复
gunzip -c backup_20231201_120000.sql.gz | psql -h localhost -U bookmarks_user -d bookmarks_db
```

### 3. 自动化备份脚本

```bash
#!/bin/bash
# backup_database.sh

DB_NAME="bookmarks_db"
DB_USER="bookmarks_user"
BACKUP_DIR="/var/backups/postgresql"
DATE=$(date +%Y%m%d_%H%M%S)
RETENTION_DAYS=30

# 创建备份目录
mkdir -p $BACKUP_DIR

# 执行备份
pg_dump -h localhost -U $DB_USER -d $DB_NAME | gzip > $BACKUP_DIR/backup_$DATE.sql.gz

# 删除旧备份
find $BACKUP_DIR -name "backup_*.sql.gz" -mtime +$RETENTION_DAYS -delete

echo "Backup completed: backup_$DATE.sql.gz"
```

## 安全配置

### 1. 用户权限

```sql
-- 创建应用用户
CREATE USER bookmarks_app WITH PASSWORD 'secure_password';

-- 授予必要权限
GRANT CONNECT ON DATABASE bookmarks_db TO bookmarks_app;
GRANT USAGE ON SCHEMA public TO bookmarks_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO bookmarks_app;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO bookmarks_app;

-- 创建只读用户（用于报表和分析）
CREATE USER bookmarks_readonly WITH PASSWORD 'readonly_password';
GRANT CONNECT ON DATABASE bookmarks_db TO bookmarks_readonly;
GRANT USAGE ON SCHEMA public TO bookmarks_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO bookmarks_readonly;
```

### 2. 行级安全策略

```sql
-- 启用行级安全
ALTER TABLE bookmarks ENABLE ROW LEVEL SECURITY;
ALTER TABLE collections ENABLE ROW LEVEL SECURITY;
ALTER TABLE tags ENABLE ROW LEVEL SECURITY;

-- 创建安全策略
CREATE POLICY bookmarks_user_policy ON bookmarks
    FOR ALL TO bookmarks_app
    USING (user_id = current_setting('app.current_user_id', true)::UUID);

CREATE POLICY collections_user_policy ON collections
    FOR ALL TO bookmarks_app
    USING (user_id = current_setting('app.current_user_id', true)::UUID);

CREATE POLICY tags_user_policy ON tags
    FOR ALL TO bookmarks_app
    USING (user_id = current_setting('app.current_user_id', true)::UUID);
```

## 监控和维护

### 1. 性能监控查询

```sql
-- 查看慢查询
SELECT 
    query,
    calls,
    total_time,
    mean_time,
    rows
FROM pg_stat_statements
ORDER BY mean_time DESC
LIMIT 10;

-- 查看表大小
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- 查看索引使用情况
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
ORDER BY idx_scan DESC;
```

### 2. 维护脚本

```sql
-- 更新表统计信息
ANALYZE;

-- 重建索引（如果需要）
REINDEX DATABASE bookmarks_db;

-- 清理过期会话
DELETE FROM user_sessions 
WHERE expires_at < NOW() OR is_active = FALSE;

-- 清理旧审计日志（保留一年）
DELETE FROM audit_logs 
WHERE created_at < NOW() - INTERVAL '1 year';
```

---

这个数据库设计提供了完整、可扩展和高性能的数据存储解决方案，支持书签应用的所有核心功能，并为未来的功能扩展预留了空间。