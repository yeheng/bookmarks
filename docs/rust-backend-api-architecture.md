# Rust 后端 API 架构设计

## 概述

本文档详细描述了书签应用的 Rust 后端 API 架构，使用 Axum 框架构建高性能、类型安全的 RESTful API。

## 技术栈

- **Web 框架**: Axum 0.7+
- **异步运行时**: Tokio
- **数据库**: PostgreSQL + SQLx
- **序列化**: Serde
- **认证**: JWT + bcrypt
- **日志**: tracing
- **配置**: config-rs
- **错误处理**: anyhow + thiserror

## 项目结构

```
backend/
├── src/
│   ├── main.rs                 # 应用入口点
│   ├── config/
│   │   ├── mod.rs
│   │   ├── database.rs         # 数据库配置
│   │   └── auth.rs             # 认证配置
│   ├── models/
│   │   ├── mod.rs
│   │   ├── user.rs             # 用户模型
│   │   ├── bookmark.rs         # 书签模型
│   │   ├── collection.rs       # 收藏夹模型
│   │   └── tag.rs              # 标签模型
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── auth.rs             # 认证处理器
│   │   ├── bookmarks.rs        # 书签处理器
│   │   ├── collections.rs      # 收藏夹处理器
│   │   └── tags.rs             # 标签处理器
│   ├── services/
│   │   ├── mod.rs
│   │   ├── auth_service.rs     # 认证服务
│   │   ├── bookmark_service.rs # 书签服务
│   │   ├── collection_service.rs # 收藏夹服务
│   │   ├── tag_service.rs      # 标签服务
│   │   └── search_service.rs   # 搜索服务
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs             # 认证中间件
│   │   ├── cors.rs             # CORS 中间件
│   │   └── logging.rs          # 日志中间件
│   ├── utils/
│   │   ├── mod.rs
│   │   ├── error.rs            # 错误处理
│   │   ├── response.rs         # 响应工具
│   │   ├── jwt.rs              # JWT 工具
│   │   └── validation.rs       # 验证工具
│   └── routes/
│       ├── mod.rs
│       ├── auth.rs             # 认证路由
│       ├── bookmarks.rs        # 书签路由
│       ├── collections.rs      # 收藏夹路由
│       └── tags.rs             # 标签路由
├── migrations/                 # 数据库迁移
├── tests/                      # 集成测试
└── Cargo.toml
```

## 核心组件设计

### 1. 应用入口 (main.rs)

```rust
use axum::{Router, middleware};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

mod config;
mod models;
mod handlers;
mod services;
mod middleware;
mod utils;
mod routes;

use config::AppConfig;
use middleware::{auth_middleware, logging_middleware};
use routes::{auth_routes, bookmark_routes, collection_routes, tag_routes};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::init();
    
    // 加载配置
    let config = AppConfig::from_env()?;
    
    // 初始化数据库连接池
    let db_pool = config.database.create_pool().await?;
    
    // 构建应用路由
    let app = Router::new()
        .nest("/api/auth", auth_routes())
        .nest("/api/bookmarks", bookmark_routes())
        .nest("/api/collections", collection_routes())
        .nest("/api/tags", tag_routes())
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
                .layer(middleware::from_fn(logging_middleware))
        )
        .with_state(db_pool);
    
    // 启动服务器
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

### 2. 配置管理 (config/mod.rs)

```rust
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expires_in: u64,
    pub refresh_token_expires_in: u64,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        config::Config::builder()
            .add_source(config::File::with_name("config"))
            .add_source(config::Environment::with_prefix("APP"))
            .build()?
            .try_deserialize()
    }
}

impl DatabaseConfig {
    pub async fn create_pool(&self) -> anyhow::Result<PgPool> {
        let pool = PgPool::connect(&self.url).await?;
        Ok(pool)
    }
}
```

### 3. 数据模型 (models/)

#### 用户模型 (models/user.rs)

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}
```

#### 书签模型 (models/bookmark.rs)

```rust
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Bookmark {
    pub id: Uuid,
    pub user_id: Uuid,
    pub collection_id: Option<Uuid>,
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub favicon_url: Option<String>,
    pub screenshot_url: Option<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub visit_count: i32,
    pub last_visited: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateBookmark {
    pub title: String,
    pub url: String,
    pub description: Option<String>,
    pub collection_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBookmark {
    pub title: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub collection_id: Option<Option<Uuid>>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct BookmarkWithTags {
    #[serde(flatten)]
    pub bookmark: Bookmark,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct BookmarkQuery {
    pub collection_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
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

    pub async fn register(&self, user_data: CreateUser, db_pool: &sqlx::PgPool) -> Result<User> {
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

    pub async fn login(&self, login_data: LoginUser, db_pool: &sqlx::PgPool) -> Result<User> {
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

#### 书签服务 (services/bookmark_service.rs)

```rust
use anyhow::Result;
use uuid::Uuid;
use sqlx::PgPool;

use crate::models::{
    Bookmark, CreateBookmark, UpdateBookmark, BookmarkWithTags, BookmarkQuery
};
use crate::utils::error::AppError;

pub struct BookmarkService;

impl BookmarkService {
    pub async fn create_bookmark(
        user_id: Uuid,
        bookmark_data: CreateBookmark,
        db_pool: &PgPool,
    ) -> Result<Bookmark> {
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
        db_pool: &PgPool,
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
        db_pool: &PgPool,
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
        db_pool: &PgPool,
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
        db_pool: &PgPool,
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
        db_pool: &PgPool,
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
use sqlx::PgPool;

use crate::models::{CreateUser, LoginUser, UserResponse};
use crate::services::auth_service::AuthService;
use crate::utils::error::AppError;
use crate::config::AuthConfig;

pub async fn register(
    State(db_pool): State<PgPool>,
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
    State(db_pool): State<PgPool>,
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

pub fn bookmark_routes() -> Router<AuthConfig> {
    Router::new()
        .route("/", get(get_bookmarks))
        .route("/", post(create_bookmark))
        .route("/{:id}", get(get_bookmark))
        .route("/{:id}", put(update_bookmark))
        .route("/{:id}", delete(delete_bookmark))
        .route("/{:id}/visit", post(increment_visit_count))
        .route("/import", post(import_bookmarks))
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
    use sqlx::PgPool;

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

    async fn setup_test_db() -> PgPool {
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

COPY --from=builder /app/target/release/bookmarks-app /usr/local/bin/

EXPOSE 3000

CMD ["bookmarks-app"]
```

### Cargo.toml 依赖

```toml
[package]
name = "bookmarks-app"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "migrate"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
bcrypt = "0.15"
jsonwebtoken = "9.0"
config = "0.14"
anyhow = "1.0"
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

[dev-dependencies]
tokio-test = "0.4"
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

---

这个 Rust 后端 API 架构提供了完整的书签应用后端功能，具有高性能、类型安全和良好的可维护性。