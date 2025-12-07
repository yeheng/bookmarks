# Rust 后端 API 架构设计

## 概述

本文档详细描述了多资源聚合系统的 Rust 后端 API 架构，使用 Axum 框架构建高性能、类型安全的 RESTful API。系统支持链接、文件、笔记等多种类型资源的统一管理。

## 技术栈

- **Web 框架**: Axum 0.8+
- **异步运行时**: Tokio
- **数据库**: SQLite + SQLx
- **全文搜索**: SQLite FTS5 (支持中英文混合搜索)
- **序列化**: Serde
- **认证**: JWT + bcrypt
- **日志**: tracing
- **配置**: config-rs + dotenv
- **错误处理**: anyhow + thiserror
- **中文分词**: jieba-rs (可选功能)

## 项目结构

```
backend/
├── src/
│   ├── main.rs                 # 应用入口点
│   ├── lib.rs                  # 库入口点
│   ├── state.rs                # 应用状态管理
│   ├── config/
│   │   ├── mod.rs
│   │   ├── app.rs              # 应用配置
│   │   ├── auth.rs             # 认证配置
│   │   ├── database.rs         # 数据库配置
│   │   └── loader.rs           # 配置加载器
│   ├── models/
│   │   ├── mod.rs
│   │   ├── resource.rs         # 资源模型
│   │   ├── collection.rs       # 收藏夹模型
│   │   ├── pagination.rs       # 分页模型
│   │   ├── search.rs           # 搜索模型
│   │   ├── stats.rs            # 统计模型
│   │   ├── tag.rs              # 标签模型
│   │   └── user.rs             # 用户模型
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── auth.rs             # 认证处理器
│   │   ├── resources.rs        # 资源处理器
│   │   ├── collections.rs      # 收藏夹处理器
│   │   ├── search.rs           # 搜索处理器
│   │   ├── stats.rs            # 统计处理器
│   │   └── tags.rs             # 标签处理器
│   ├── services/
│   │   ├── mod.rs
│   │   ├── auth_service.rs     # 认证服务
│   │   ├── resource_service.rs # 资源服务
│   │   ├── collection_service.rs # 收藏夹服务
│   │   ├── indexer_service.rs  # 索引服务
│   │   ├── maintenance_service.rs # 维护服务
│   │   ├── search_service.rs   # 搜索服务
│   │   ├── stats_service.rs    # 统计服务
│   │   └── tag_service.rs      # 标签服务
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs             # 认证中间件
│   │   ├── cors.rs             # CORS 中间件
│   │   └── logging.rs          # 日志中间件
│   ├── utils/
│   │   ├── mod.rs
│   │   ├── error.rs            # 错误处理
│   │   ├── jwt.rs              # JWT 工具
│   │   ├── response.rs         # 响应工具
│   │   ├── segmenter.rs        # 分词器
│   │   └── validation.rs       # 验证工具
│   └── routes/
│       ├── mod.rs
│       ├── auth.rs             # 认证路由
│       ├── resources.rs        # 资源路由
│       ├── collections.rs      # 收藏夹路由
│       ├── search.rs           # 搜索路由
│       ├── stats.rs            # 统计路由
│       └── tags.rs             # 标签路由
├── migrations/                 # 数据库迁移
│   ├── 20250104000001_create_tables.sql
│   ├── 20250104000002_optimize_indexes.sql
│   ├── 20250104000003_setup_fts5.sql
│   └── 20250104000004_seed_data.sql
├── tests/                      # 集成测试
│   ├── fts_config_integration.rs
│   └── segmentation_integration.rs
├── config/                     # 配置文件
│   ├── default.toml
│   ├── development.toml
│   └── production.toml
├── .env.example                # 环境变量示例
└── Cargo.toml
```

## 核心组件设计

### 1. 应用入口 (main.rs)

```rust
use axum::{middleware as mw, Router};
use axum_jwt_auth::Decoder;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{self, EnvFilter};

mod config;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod state;
mod utils;

use config::AppConfig;
use middleware::{auth_middleware, logging_middleware};
use routes::{
    ano_routes, auth_routes, resource_routes, collection_routes, search_routes, stats_routes,
    tag_routes,
};
use state::AppState;
use utils::jwt::{JWTService, JwtClaims};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize tracing with a sensible default when RUST_LOG isn't set
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_target(true)
        .with_env_filter(env_filter)
        .init();

    // Load configuration with support for config files and environment variables
    let config = AppConfig::load()?;

    // Initialize database connection pool
    let db_pool = config.database.create_pool().await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    // 检查并重建 FTS 索引（如果需要）
    services::check_and_rebuild_fts(db_pool.clone()).await?;

    // Initialize shared JWT decoder for middleware
    let jwt_decoder: Decoder<JwtClaims> = Arc::new(JWTService::new(config.auth.jwt_secret.clone()));

    let app_state = AppState::new(db_pool.clone(), jwt_decoder);

    // Protected routes requiring authentication
    let protected_routes = Router::new()
        .nest("/api/resources", resource_routes())
        .nest("/api/collections", collection_routes())
        .nest("/api/tags", tag_routes())
        .nest("/api/search", search_routes())
        .nest("/api/stats", stats_routes())
        .nest("/api/auth", auth_routes())
        .layer(mw::from_fn_with_state(app_state.clone(), auth_middleware));

    // Build application router
    let app = Router::new()
        .nest("/api/auth", ano_routes())
        .merge(protected_routes)
        .layer(middleware::cors::cors_layer())
        .layer(mw::from_fn(logging_middleware))
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

### 2. 应用状态管理 (state.rs)

```rust
use axum::extract::FromRef;
use axum_jwt_auth::Decoder;
use sqlx::SqlitePool;

use crate::utils::jwt::JwtClaims;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
    pub jwt_decoder: Decoder<JwtClaims>,
}

impl AppState {
    pub fn new(
        db_pool: SqlitePool,
        jwt_decoder: Decoder<JwtClaims>,
    ) -> Self {
        Self {
            db_pool,
            jwt_decoder,
        }
    }
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.db_pool.clone()
    }
}

impl FromRef<AppState> for Decoder<JwtClaims> {
    fn from_ref(state: &AppState) -> Self {
        state.jwt_decoder.clone()
    }
}
```

### 3. 配置管理 (config/mod.rs)

配置系统支持 TOML 文件和环境变量，具有分层的配置结构：

- `default.toml`: 默认配置
- `development.toml`: 开发环境配置
- `production.toml`: 生产环境配置

配置加载优先级：环境变量 > 环境特定配置文件 > 默认配置文件。

### 3. 数据模型 (models/)

#### 用户模型 (models/user.rs)

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub email_verification_token: Option<String>,
    pub password_reset_token: Option<String>,
    pub password_reset_expires_at: Option<i64>,
    pub last_login_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub last_login_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            avatar_url: user.avatar_url,
            is_active: user.is_active,
            email_verified: user.email_verified,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}
```

#### 资源模型 (models/resource.rs)

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Resource {
    pub id: Uuid,
    pub user_id: Uuid,
    pub collection_id: Option<Uuid>,
    pub title: String,
    pub url: Option<String>,
    pub description: Option<String>,
    pub resource_type: String, // 'link', 'file', 'note'
    pub favicon_url: Option<String>,
    pub screenshot_url: Option<String>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub file_type: Option<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub visit_count: i32,
    pub last_visited: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateResource {
    pub title: String,
    pub url: Option<String>,
    pub description: Option<String>,
    pub resource_type: String,
    pub collection_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub file_path: Option<String>,
    pub file_content: Option<String>, // for notes
}

#[derive(Debug, Deserialize)]
pub struct UpdateResource {
    pub title: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub resource_type: Option<String>,
    pub collection_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
    pub file_path: Option<String>,
    pub file_content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResourceWithTags {
    #[serde(flatten)]
    pub resource: Resource,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceQuery {
    pub collection_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub resource_type: Option<String>,
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
```

### 4. 服务层 (services/)

#### 认证服务 (services/auth_service.rs)

```rust
use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::models::{User, CreateUser, LoginUser};
use crate::utils::error::AppError;
use crate::config::AuthConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // 用户 ID
    pub exp: usize,   // 过期时间
    pub iat: usize,   // 签发时间
}

pub struct AuthService {
    config: AuthConfig,
}

impl AuthService {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }

    pub async fn register(&self, user_data: CreateUser, db_pool: &sqlx::SqlitePool) -> Result<User> {
        // 检查用户名和邮箱是否已存在
        let existing = sqlx::query!(
            "SELECT id FROM users WHERE username = $1 OR email = $2",
            user_data.username,
            user_data.email
        )
        .fetch_optional(db_pool)
        .await?;

        if existing.is_some() {
            return Err(AppError::Conflict("用户名或邮箱已存在".to_string()).into());
        }

        // 加密密码
        let password_hash = hash(&user_data.password, DEFAULT_COST)?;

        // 创建用户
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            user_data.username,
            user_data.email,
            password_hash
        )
        .fetch_one(db_pool)
        .await?;

        Ok(user)
    }

    pub async fn login(&self, login_data: LoginUser, db_pool: &sqlx::SqlitePool) -> Result<User> {
        // 查找用户
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE email = $1",
            login_data.email
        )
        .fetch_optional(db_pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("邮箱或密码错误".to_string()))?;

        // 验证密码
        let is_valid = verify(&login_data.password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::Unauthorized("邮箱或密码错误".to_string()).into());
        }

        Ok(user)
    }

    pub fn generate_access_token(&self, user_id: Uuid) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::minutes(self.config.jwt_expires_in as i64);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    pub fn generate_refresh_token(&self, user_id: Uuid) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::days(self.config.refresh_token_expires_in as i64);

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.config.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<Uuid> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.config.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Unauthorized("无效的 token".to_string()))?;

        Ok(user_id)
    }
}
```

#### 资源服务 (services/resource_service.rs)

```rust
use anyhow::Result;
use uuid::Uuid;
use sqlx::SqlitePool;

use crate::models::{
    Resource, CreateResource, UpdateResource, ResourceWithTags, ResourceQuery
};
use crate::utils::error::AppError;

pub struct ResourceService;

impl ResourceService {
    pub async fn create_resource(
        user_id: Uuid,
        resource_data: CreateResource,
        db_pool: &SqlitePool,
    ) -> Result<Resource> {
        // 开始事务
        let mut tx = db_pool.begin().await?;

        // 创建书签
        let bookmark = sqlx::query_as!(
            Bookmark,
            r#"
            INSERT INTO bookmarks (user_id, collection_id, title, url, description, is_favorite)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            user_id,
            bookmark_data.collection_id,
            bookmark_data.title,
            bookmark_data.url,
            bookmark_data.description,
            bookmark_data.is_favorite.unwrap_or(false)
        )
        .fetch_one(&mut *tx)
        .await?;

        // 处理标签
        if let Some(tags) = bookmark_data.tags {
            for tag_name in tags {
                // 确保标签存在
                let tag = sqlx::query!(
                    r#"
                    INSERT INTO tags (user_id, name)
                    VALUES ($1, $2)
                    ON CONFLICT (user_id, name) DO UPDATE SET name = EXCLUDED.name
                    RETURNING id
                    "#,
                    user_id,
                    tag_name
                )
                .fetch_one(&mut *tx)
                .await?;

                // 关联书签和标签
                sqlx::query!(
                    "INSERT INTO bookmark_tags (bookmark_id, tag_id) VALUES ($1, $2)",
                    bookmark.id,
                    tag.id
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        // 提交事务
        tx.commit().await?;

        Ok(bookmark)
    }

    pub async fn get_bookmarks(
        user_id: Uuid,
        query: BookmarkQuery,
        db_pool: &SqlitePool,
    ) -> Result<Vec<BookmarkWithTags>> {
        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let mut sql = r#"
            SELECT 
                b.*,
                COALESCE(
                    array_agg(t.name) FILTER (WHERE t.name IS NOT NULL),
                    ARRAY[]::VARCHAR[]
                ) as tags
            FROM bookmarks b
            LEFT JOIN bookmark_tags bt ON b.id = bt.bookmark_id
            LEFT JOIN tags t ON bt.tag_id = t.id
            WHERE b.user_id = $1
        "#.to_string();

        let mut param_count = 1;
        let mut params: Vec<Box<dyn sqlx::database::HasArguments<sqlx::Postgres> + Send>> = Vec::new();

        // 添加过滤条件
        if let Some(collection_id) = query.collection_id {
            param_count += 1;
            sql.push_str(&format!(" AND b.collection_id = ${}", param_count));
        }

        if let Some(is_favorite) = query.is_favorite {
            param_count += 1;
            sql.push_str(&format!(" AND b.is_favorite = ${}", param_count));
        }

        if let Some(is_archived) = query.is_archived {
            param_count += 1;
            sql.push_str(&format!(" AND b.is_archived = ${}", param_count));
        }

        if let Some(search_term) = query.search {
            param_count += 1;
            sql.push_str(&format!(
                " AND (to_tsvector('english', b.title || ' ' || COALESCE(b.description, '')) @@ plainto_tsquery('english', ${}))",
                param_count
            ));
        }

        sql.push_str(&format!(
            " GROUP BY b.id ORDER BY b.created_at DESC LIMIT ${} OFFSET ${}",
            param_count + 1,
            param_count + 2
        ));

        let bookmarks = sqlx::query_as(&sql)
            .bind(user_id)
            .bind(query.collection_id)
            .bind(query.is_favorite)
            .bind(query.is_archived)
            .bind(query.search)
            .bind(limit)
            .bind(offset)
            .fetch_all(db_pool)
            .await?;

        Ok(bookmarks)
    }

    pub async fn get_bookmark_by_id(
        user_id: Uuid,
        bookmark_id: Uuid,
        db_pool: &SqlitePool,
    ) -> Result<Option<BookmarkWithTags>> {
        let bookmark = sqlx::query_as!(
            BookmarkWithTags,
            r#"
            SELECT 
                b.*,
                COALESCE(
                    array_agg(t.name) FILTER (WHERE t.name IS NOT NULL),
                    ARRAY[]::VARCHAR[]
                ) as tags
            FROM bookmarks b
            LEFT JOIN bookmark_tags bt ON b.id = bt.bookmark_id
            LEFT JOIN tags t ON bt.tag_id = t.id
            WHERE b.id = $1 AND b.user_id = $2
            GROUP BY b.id
            "#,
            bookmark_id,
            user_id
        )
        .fetch_optional(db_pool)
        .await?;

        Ok(bookmark)
    }

    pub async fn update_bookmark(
        user_id: Uuid,
        bookmark_id: Uuid,
        update_data: UpdateBookmark,
        db_pool: &SqlitePool,
    ) -> Result<Option<Bookmark>> {
        // 构建动态更新查询
        let mut updates = Vec::new();
        let mut params = Vec::new();
        let mut param_count = 0;

        if let Some(title) = update_data.title {
            param_count += 1;
            updates.push(format!("title = ${}", param_count));
            params.push(title);
        }

        if let Some(url) = update_data.url {
            param_count += 1;
            updates.push(format!("url = ${}", param_count));
            params.push(url);
        }

        if let Some(description) = update_data.description {
            param_count += 1;
            updates.push(format!("description = ${}", param_count));
            params.push(description);
        }

        if let Some(collection_id) = update_data.collection_id {
            param_count += 1;
            updates.push(format!("collection_id = ${}", param_count));
            params.push(collection_id);
        }

        if let Some(is_favorite) = update_data.is_favorite {
            param_count += 1;
            updates.push(format!("is_favorite = ${}", param_count));
            params.push(is_favorite);
        }

        if let Some(is_archived) = update_data.is_archived {
            param_count += 1;
            updates.push(format!("is_archived = ${}", param_count));
            params.push(is_archived);
        }

        if updates.is_empty() {
            return Err(AppError::BadRequest("没有提供更新字段".to_string()).into());
        }

        updates.push("updated_at = NOW()".to_string());

        let sql = format!(
            "UPDATE bookmarks SET {} WHERE id = ${} AND user_id = ${} RETURNING *",
            updates.join(", "),
            param_count + 1,
            param_count + 2
        );

        let mut query = sqlx::query_as::<_, Bookmark>(&sql);
        for param in params {
            query = query.bind(param);
        }
        query = query.bind(bookmark_id).bind(user_id);

        let bookmark = query.fetch_optional(db_pool).await?;

        // 处理标签更新
        if let Some(tags) = update_data.tags {
            // 删除现有标签关联
            sqlx::query!("DELETE FROM bookmark_tags WHERE bookmark_id = $1", bookmark_id)
                .execute(db_pool)
                .await?;

            // 添加新标签关联
            for tag_name in tags {
                let tag = sqlx::query!(
                    r#"
                    INSERT INTO tags (user_id, name)
                    VALUES ($1, $2)
                    ON CONFLICT (user_id, name) DO UPDATE SET name = EXCLUDED.name
                    RETURNING id
                    "#,
                    user_id,
                    tag_name
                )
                .fetch_one(db_pool)
                .await?;

                sqlx::query!(
                    "INSERT INTO bookmark_tags (bookmark_id, tag_id) VALUES ($1, $2)",
                    bookmark_id,
                    tag.id
                )
                .execute(db_pool)
                .await?;
            }
        }

        Ok(bookmark)
    }

    pub async fn delete_bookmark(
        user_id: Uuid,
        bookmark_id: Uuid,
        db_pool: &SqlitePool,
    ) -> Result<bool> {
        let result = sqlx::query!(
            "DELETE FROM bookmarks WHERE id = $1 AND user_id = $2",
            bookmark_id,
            user_id
        )
        .execute(db_pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn increment_visit_count(
        bookmark_id: Uuid,
        db_pool: &SqlitePool,
    ) -> Result<()> {
        sqlx::query!(
            "UPDATE bookmarks SET visit_count = visit_count + 1, last_visited = NOW() WHERE id = $1",
            bookmark_id
        )
        .execute(db_pool)
        .await?;

        Ok(())
    }
}
```

### 5. 处理器层 (handlers/)

#### 认证处理器 (handlers/auth.rs)

```rust
use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::models::{CreateUser, LoginUser, UserResponse};
use crate::services::auth_service::AuthService;
use crate::utils::error::AppError;
use crate::config::AuthConfig;

pub async fn register(
    State(db_pool): State<SqlitePool>,
    State(auth_config): State<AuthConfig>,
    Json(user_data): Json<CreateUser>,
) -> Result<ResponseJson<Value>, AppError> {
    let auth_service = AuthService::new(auth_config);
    
    let user = auth_service.register(user_data, &db_pool).await?;
    let access_token = auth_service.generate_access_token(user.id)?;
    let refresh_token = auth_service.generate_refresh_token(user.id)?;

    Ok(ResponseJson(json!({
        "user": UserResponse::from(user),
        "access_token": access_token,
        "refresh_token": refresh_token
    })))
}

pub async fn login(
    State(db_pool): State<SqlitePool>,
    State(auth_config): State<AuthConfig>,
    Json(login_data): Json<LoginUser>,
) -> Result<ResponseJson<Value>, AppError> {
    let auth_service = AuthService::new(auth_config);
    
    let user = auth_service.login(login_data, &db_pool).await?;
    let access_token = auth_service.generate_access_token(user.id)?;
    let refresh_token = auth_service.generate_refresh_token(user.id)?;

    Ok(ResponseJson(json!({
        "user": UserResponse::from(user),
        "access_token": access_token,
        "refresh_token": refresh_token
    })))
}

pub async fn refresh_token(
    State(auth_config): State<AuthConfig>,
    Json(body): Json<Value>,
) -> Result<ResponseJson<Value>, AppError> {
    let refresh_token = body.get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("缺少 refresh_token".to_string()))?;

    let auth_service = AuthService::new(auth_config);
    let user_id = auth_service.verify_token(refresh_token)?;
    let access_token = auth_service.generate_access_token(user_id)?;

    Ok(ResponseJson(json!({
        "access_token": access_token
    })))
}
```

### 6. 路由定义 (routes/)

```rust
use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::{self, auth::*, bookmarks::*, collections::*, tags::*};
use crate::middleware::auth_middleware;
use crate::config::AuthConfig;

pub fn auth_routes() -> Router<AuthConfig> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh_token))
}

pub fn resource_routes() -> Router<AuthConfig> {
    Router::new()
        .route("/", get(get_resources))
        .route("/", post(create_resource))
        .route("/{:id}", get(get_resource))
        .route("/{:id}", put(update_resource))
        .route("/{:id}", delete(delete_resource))
        .route("/{:id}/visit", post(increment_visit_count))
        .route("/import", post(import_resources))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn collection_routes() -> Router<AuthConfig> {
    Router::new()
        .route("/", get(get_collections))
        .route("/", post(create_collection))
        .route("/{:id}", get(get_collection))
        .route("/{:id}", put(update_collection))
        .route("/{:id}", delete(delete_collection))
        .layer(middleware::from_fn(auth_middleware))
}

pub fn tag_routes() -> Router<AuthConfig> {
    Router::new()
        .route("/", get(get_tags))
        .route("/", post(create_tag))
        .route("/{:id}", put(update_tag))
        .route("/{:id}", delete(delete_tag))
        .layer(middleware::from_fn(auth_middleware))
}
```

### 7. 错误处理 (utils/error.rs)

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("认证错误: {0}")]
    Unauthorized(String),
    
    #[error("请求冲突: {0}")]
    Conflict(String),
    
    #[error("请求错误: {0}")]
    BadRequest(String),
    
    #[error("未找到资源: {0}")]
    NotFound(String),
    
    #[error("内部服务器错误: {0}")]
    Internal(String),
    
    #[error("JWT 错误: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    #[error("密码加密错误: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(err) => {
                tracing::error!("数据库错误: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误")
            }
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.as_str()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.as_str()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.as_str()),
            AppError::Internal(msg) => {
                tracing::error!("内部错误: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误")
            }
            AppError::Jwt(_) => (StatusCode::UNAUTHORIZED, "无效的认证令牌"),
            AppError::Bcrypt(_) => (StatusCode::INTERNAL_SERVER_ERROR, "密码处理错误"),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}
```

## 中间件设计

### 认证中间件 (middleware/auth.rs)

```rust
use axum::{
    extract::{Request, State},
    http::{header::AUTHORIZATION, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::services::auth_service::AuthService;
use crate::config::AuthConfig;
use crate::utils::error::AppError;

pub async fn auth_middleware(
    State(auth_config): State<AuthConfig>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let token = auth_header.ok_or_else(|| {
        AppError::Unauthorized("缺少认证令牌".to_string())
    })?;

    let auth_service = AuthService::new(auth_config);
    let user_id = auth_service.verify_token(token)?;

    // 将用户 ID 添加到请求扩展中
    request.extensions_mut().insert(user_id);

    Ok(next.run(request).await)
}

// 用于从请求中提取用户 ID 的辅助函数
pub fn extract_user_id(request: &Request) -> Result<Uuid, AppError> {
    request
        .extensions()
        .get::<Uuid>()
        .copied()
        .ok_or_else(|| AppError::Unauthorized("用户未认证".to_string()))
}
```

## 测试策略

### 单元测试示例

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_user_registration() {
        // 设置测试数据库
        let pool = setup_test_db().await;
        
        let auth_service = AuthService::new(test_auth_config());
        let user_data = CreateUser {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };

        let user = auth_service.register(user_data, &pool).await.unwrap();
        
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    async fn setup_test_db() -> SqlitePool {
        // 创建测试数据库连接池
        todo!()
    }

    fn test_auth_config() -> AuthConfig {
        AuthConfig {
            jwt_secret: "test_secret".to_string(),
            jwt_expires_in: 15,
            refresh_token_expires_in: 7,
        }
    }
}
```

## 部署配置

### Dockerfile

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/resources-app /usr/local/bin/

EXPOSE 3000

CMD ["resources-app"]
```

### Cargo.toml 依赖

```toml
[package]
name = "resources-api"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web framework
axum = { version = "0.8", features = ["multipart"] }
tokio = { version = "1.0", features = ["full"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace"] }

# Database
sqlx = { version = "0.8", features = [
    "runtime-tokio-rustls",
    "sqlite",
    "migrate",
] }

async-trait = "0.1"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Time/Date
chrono = { version = "0.4", features = ["serde"] }

# Authentication
jsonwebtoken = { version = "10.2.0", features = ["rust_crypto"] }
bcrypt = "0.17"
axum-jwt-auth = "0.6"

# Configuration
config = "0.15"

# Error handling
anyhow = "1.0"
thiserror = "2.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Environment variables
dotenv = "0.15"

# Validation
regex = "1.0"

# Lazy initialization
once_cell = "1.20"

# Full-text search (Chinese word segmentation for FTS5)
jieba-rs = { version = "0.8", optional = true }

[features]
default = []
jieba = ["jieba-rs"]

[dev-dependencies]
tokio-test = "0.4"
tempfile = "3.0"
```

## 性能优化

1. **连接池优化**: 合理配置数据库连接池大小
2. **查询优化**: 使用索引和预编译语句
3. **缓存策略**: 对频繁查询的数据实现缓存
4. **异步处理**: 充分利用 Rust 的异步特性
5. **响应压缩**: 启用 HTTP 响应压缩

## 安全措施

1. **输入验证**: 对所有用户输入进行严格验证
2. **SQL 注入防护**: 使用参数化查询
3. **认证安全**: 强密码策略和 JWT 安全配置
4. **CORS 配置**: 合理配置跨域访问策略
5. **错误处理**: 避免泄露敏感信息

## 全文搜索架构

### FTS5 配置

项目使用 SQLite FTS5 进行全文搜索，支持中英文混合搜索：

```sql
CREATE VIRTUAL TABLE resources_fts USING fts5(
    title, 
    description,
    tags,
    url,
    tokenize = 'unicode61 remove_diacritics 2'
);
```

### 搜索服务

搜索服务提供以下功能：
- 全文搜索（标题、描述、标签、URL）
- 搜索建议
- 高亮显示
- 按相关性排序

### 中文分词支持

可选的 jieba 分词支持：
```bash
# 启用 jieba 功能
cargo run --features jieba
```

## 性能优化

1. **数据库索引优化**：
   - 基础索引用于常见查询
   - 部分索引用于特定场景
   - 复合索引用于复杂查询

2. **连接池配置**：
   - SQLite 连接池优化
   - WAL 模式提高并发性能

3. **搜索优化**：
   - FTS5 虚拟表
   - 触发器自动同步
   - 索引重建机制

## 安全特性

1. **认证安全**：
   - JWT 令牌认证
   - bcrypt 密码哈希
   - 令牌自动刷新

2. **数据安全**：
   - SQL 注入防护
   - 输入验证
   - 错误信息过滤

3. **CORS 配置**：
   - 可配置的跨域策略
   - 开发/生产环境分离

---

这个 Rust 后端 API 架构提供了完整的书签应用后端功能，具有高性能、类型安全和良好的可维护性。特别针对 SQLite 和全文搜索进行了优化，支持中英文混合搜索，适合中小型应用和快速开发迭代。