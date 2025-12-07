# 开发者指南

本指南为开发者提供参与多资源聚合系统开发的详细信息，包括代码规范、开发流程、架构说明和最佳实践。系统支持链接、文件、笔记等多种类型资源的统一管理。

## 📋 目录

- [快速开始](#快速开始)
- [项目架构](#项目架构)
- [开发流程](#开发流程)
- [代码规范](#代码规范)
- [测试指南](#测试指南)
- [API 开发](#api-开发)
- [前端开发](#前端开发)
- [数据库开发](#数据库开发)
- [调试技巧](#调试技巧)
- [性能优化](#性能优化)
- [贡献指南](#贡献指南)

## 🚀 快速开始

### 环境准备

1. **安装基础工具**

```bash
# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 20

# Docker (可选)
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
```

2. **克隆项目**

```bash
git clone <repository-url>
cd bookmarks
```

3. **启动开发环境**

```bash
# 后端
cd backend
cargo run

# 前端
cd frontend
npm run dev
```

### 开发工具配置

#### VS Code 配置

推荐安装以下扩展：

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "Vue.volar",
    "bradlc.vscode-tailwindcss",
    "esbenp.prettier-vscode",
    "ms-vscode.vscode-typescript-next"
  ]
}
```

#### IDE 配置

创建 `.vscode/settings.json`：

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "typescript.preferences.importModuleSpecifier": "relative",
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  }
}
```

## 🏗️ 项目架构

### 整体架构

```
┌─────────────────┐    HTTP     ┌─────────────────┐
│   Vue.js 3      │ ◄─────────► │   Rust + Axum   │
│   Frontend      │             │   Backend       │
│   + Reka UI     │             │   + SQLite      │
└─────────────────┘             └─────────────────┘
         │                               │
         │                               │
    Vite Dev Server                 SQLite Database
                                     + FTS5 Search
```

### 后端架构

```
┌─────────────────────────────────────────────────┐
│                    Axum Web Server              │
├─────────────────────────────────────────────────┤
│  Middleware Layer                               │
│  ├─ Auth Middleware                            │
│  ├─ CORS Middleware                            │
│  └─ Logging Middleware                         │
├─────────────────────────────────────────────────┤
│  Handler Layer                                  │
│  ├─ Auth Handlers                              │
│  ├─ Resource Handlers                          │
│  ├─ Collection Handlers                        │
│  └─ Tag Handlers                               │
├─────────────────────────────────────────────────┤
│  Service Layer                                  │
│  ├─ Auth Service                               │
│  ├─ Resource Service                           │
│  └─ Search Service                             │
├─────────────────────────────────────────────────┤
│  Model Layer                                    │
│  ├─ User Model                                 │
│  ├─ Resource Model                             │
│  ├─ Collection Model                           │
│  └─ Tag Model                                  │
├─────────────────────────────────────────────────┤
│  Database Layer                                 │
│  └─ SQLite + SQLx                              │
└─────────────────────────────────────────────────┘
```

### 前端架构

```
┌─────────────────────────────────────────────────┐
│                 Vue.js 3 Application            │
├─────────────────────────────────────────────────┤
│  Presentation Layer                             │
│  ├─ Views (Pages)                              │
│  ├─ Components                                 │
│  └─ UI Components (Reka UI)                    │
├─────────────────────────────────────────────────┤
│  Business Layer                                 │
│  ├─ Composables                                │
│  ├─ Services                                   │
│  └─ API Client                                 │
├─────────────────────────────────────────────────┤
│  State Management                               │
│  └─ Pinia Stores                               │
├─────────────────────────────────────────────────┤
│  Routing Layer                                  │
│  └─ Vue Router                                 │
├─────────────────────────────────────────────────┤
│  Utility Layer                                  │
│  ├─ Utils                                      │
│  ├─ Types                                      │
│  └─ Constants                                  │
└─────────────────────────────────────────────────┘
```

## 🔄 开发流程

### Git 工作流

1. **分支策略**

```bash
main          # 主分支，生产环境代码
develop       # 开发分支，集成新功能
feature/*     # 功能分支
hotfix/*      # 热修复分支
release/*     # 发布分支
```

2. **提交规范**

```bash
# 功能开发
git commit -m "feat: add resource search functionality"

# 问题修复
git commit -m "fix: resolve authentication issue"

# 文档更新
git commit -m "docs: update API documentation"

# 代码重构
git commit -m "refactor: optimize database queries"
```

### 开发步骤

1. **创建功能分支**

```bash
git checkout develop
git pull origin develop
git checkout -b feature/resource-search
```

2. **开发和测试**

```bash
# 后端开发
cd backend
cargo test

# 前端开发
cd frontend
npm run test
```

3. **提交代码**

```bash
git add .
git commit -m "feat: implement resource search"
git push origin feature/resource-search
```

4. **创建 Pull Request**

- 填写详细的 PR 描述
- 确保所有测试通过
- 请求代码审查

### 代码审查清单

- [ ] 代码符合项目规范
- [ ] 包含适当的测试
- [ ] 文档已更新
- [ ] 没有硬编码的配置
- [ ] 错误处理完善
- [ ] 性能考虑合理
- [ ] 支持多种资源类型（链接、文件、笔记）

## 📝 代码规范

### Rust 代码规范

#### 命名约定

```rust
// 变量和函数：snake_case
let user_id = 123;
fn get_user_by_id(id: i32) -> Option<User> { }

// 常量：SCREAMING_SNAKE_CASE
const MAX_RETRY_COUNT: u32 = 3;

// 类型和结构体：PascalCase
struct UserService {
    client: reqwest::Client,
}

// 枚举：PascalCase
enum UserRole {
    Admin,
    User,
}
```

#### 代码组织

```rust
// 文件结构
mod models;      // 数据模型
mod handlers;    // HTTP 处理器
mod services;    // 业务逻辑
mod utils;       // 工具函数
mod middleware;  // 中间件

// 使用声明
use std::collections::HashMap;
use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
```

#### 错误处理

```rust
// 使用 Result 类型
fn get_user(id: i32) -> Result<Option<User>, AppError> {
    // 实现
}

// 自定义错误类型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("User not found")]
    UserNotFound,
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
```

#### 文档注释

```rust
/// 获取用户信息
/// 
/// # Arguments
/// 
/// * `id` - 用户ID
/// 
/// # Returns
/// 
/// 返回用户信息或错误
/// 
/// # Examples
/// 
/// ```
/// let user = get_user(1)?;
/// println!("User: {}", user.username);
/// ```
pub fn get_user(id: i32) -> Result<Option<User>, AppError> {
    // 实现
}
```

### TypeScript 代码规范

#### 命名约定

```typescript
// 变量和函数：camelCase
const userId = 123;
function getUserById(id: number): User | null {
  // 实现
}

// 常量：SCREAMING_SNAKE_CASE
const API_BASE_URL = 'http://localhost:3000/api';

// 类型和接口：PascalCase
interface User {
  id: number;
  username: string;
}

class UserService {
  private client: HttpClient;
}
```

#### 组件规范

```vue
<!-- 组件命名：PascalCase -->
<template>
  <div class="user-card">
    <h3>{{ user.username }}</h3>
    <p>{{ user.email }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { User } from '@/types'

interface Props {
  user: User
  showEmail?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showEmail: true
})

const displayName = computed(() => {
  return props.user.username.toUpperCase()
})
</script>

<style scoped>
.user-card {
  @apply p-4 border rounded-lg;
}
</style>
```

#### 类型定义

```typescript
// 基础类型
interface User {
  id: number
  username: string
  email: string
  created_at: string
}

// 泛型类型
interface ApiResponse<T> {
  success: boolean
  data: T
  message?: string
}

// 联合类型
type SortOrder = 'asc' | 'desc'

// 工具类型
type PartialUser = Partial<User>
type UserWithoutId = Omit<User, 'id'>
```

## 🧪 测试指南

### 后端测试

#### 单元测试

```rust
// src/services/user_service_test.rs
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_create_user() {
        let pool = create_test_pool().await;
        let service = UserService::new(pool);
        
        let user = service.create_user(
            "testuser",
            "test@example.com",
            "password123"
        ).await.unwrap();
        
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }
}
```

#### 集成测试

```rust
// tests/integration_test.rs
use axum_test::TestServer;

#[tokio::test]
async fn test_user_registration() {
    let app = create_app().await;
    let server = TestServer::new(app).unwrap();
    
    let response = server
        .post("/api/auth/register")
        .json(&serde_json::json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "password123"
        }))
        .await;
    
    assert_eq!(response.status_code(), 200);
}
```

#### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test user_service

# 显示测试输出
cargo test -- --nocapture

# 运行基准测试
cargo bench
```

### 前端测试

#### 组件测试

```typescript
// tests/components/UserCard.test.ts
import { describe, it, expect } from 'vitest'
import { mount } from '@vue/test-utils'
import UserCard from '@/components/UserCard.vue'

describe('UserCard', () => {
  it('renders user information correctly', () => {
    const user = {
      id: 1,
      username: 'testuser',
      email: 'test@example.com'
    }
    
    const wrapper = mount(UserCard, {
      props: { user }
    })
    
    expect(wrapper.text()).toContain('testuser')
    expect(wrapper.text()).toContain('test@example.com')
  })
})
```

#### API 测试

```typescript
// tests/services/api.test.ts
import { describe, it, expect, beforeEach } from 'vitest'
import { apiService } from '@/services/api'

describe('API Service', () => {
  beforeEach(() => {
    // Mock fetch
    global.fetch = vi.fn()
  })
  
  it('should login successfully', async () => {
    const mockResponse = {
      success: true,
      data: {
        user: { id: 1, username: 'testuser' },
        access_token: 'mock-token'
      }
    }
    
    vi.mocked(fetch).mockResolvedValue({
      ok: true,
      json: async () => mockResponse
    } as Response)
    
    const result = await apiService.login({
      email: 'test@example.com',
      password: 'password123'
    })
    
    expect(result.user.username).toBe('testuser')
  })
})
```

#### 运行测试

```bash
# 运行所有测试
npm run test

# 运行特定测试文件
npm run test UserCard.test.ts

# 监听模式
npm run test -- --watch

# 覆盖率报告
npm run test -- --coverage
```

## 🔌 API 开发

### 创建新的 API 端点

#### 1. 定义路由

```rust
// src/routes/stats.rs
use axum::{
    routing::get,
    Router,
};
use crate::handlers::stats::get_user_stats;
use crate::state::AppState;

pub fn stats_routes() -> Router<AppState> {
    Router::new()
        .route("/user", get(get_user_stats))
        .route("/resources", get(get_resource_stats))
}
```

#### 2. 实现处理器

```rust
// src/handlers/stats.rs
use axum::{extract::State, response::Json};
use serde_json::json;
use crate::state::AppState;
use crate::utils::error::AppError;

pub async fn get_user_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = 1; // 从认证中间件获取
    
    let total_resources = sqlx::query_scalar!(
    "SELECT COUNT(*) FROM resources WHERE user_id = ?",
        user_id
    )
    .fetch_one(&state.db_pool)
    .await?;
    
    Ok(Json(json!({
        "total_resources": total_resources,
        "total_collections": 0,
        "total_tags": 0
    })))
}
```

#### 3. 注册路由

```rust
// src/main.rs
use crate::routes::{auth_routes, resource_routes, stats_routes};

// 在主路由中添加
let protected_routes = Router::new()
    .nest("/api/resources", resource_routes())
    .nest("/api/stats", stats_routes())
    .layer(mw::from_fn_with_state(app_state.clone(), auth_middleware));
```

### 错误处理

```rust
// src/utils/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Not found")]
    NotFound,
    
    #[error("Unauthorized")]
    Unauthorized,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(err) => {
                tracing::error!("Database error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource not found"),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
        };
        
        let body = Json(json!({
            "success": false,
            "error": {
                "message": error_message,
                "code": status.as_u16()
            }
        }));
        
        (status, body).into_response()
    }
}
```

## 🎨 前端开发

### 创建新组件

#### 1. 组件结构

```vue
<!-- src/components/ResourceCard.vue -->
<template>
  <div class="resource-card" @click="handleClick">
    <div class="resource-type">
      <span class="type-badge">{{ resourceTypeLabel }}</span>
    </div>
    <h3 class="resource-title">{{ resource.title }}</h3>
    <p v-if="resource.url" class="resource-url">{{ resource.url }}</p>
    <p v-if="resource.description" class="resource-description">
      {{ resource.description }}
    </p>
    <div class="resource-tags">
      <span 
        v-for="tag in resource.tags" 
        :key="tag.id"
        class="tag"
      >
        {{ tag.name }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Resource } from '@/types'

interface Props {
  resource: Resource
  clickable?: boolean
}

interface Emits {
  click: [resource: Resource]
}

const props = withDefaults(defineProps<Props>(), {
  clickable: true
})

const emit = defineEmits<Emits>()

const resourceTypeLabel = computed(() => {
  const labels = {
    link: '链接',
    file: '文件',
    note: '笔记'
  }
  return labels[props.resource.resource_type] || '其他'
})

const handleClick = () => {
  if (props.clickable) {
    emit('click', props.resource)
  }
}
</script>

<style scoped>
.resource-card {
  @apply p-4 border rounded-lg hover:shadow-md transition-shadow cursor-pointer;
}

.resource-type {
  @apply mb-2;
}

.type-badge {
  @apply px-2 py-1 bg-blue-100 text-blue-800 rounded text-xs font-medium;
}

.resource-title {
  @apply font-semibold text-lg mb-2;
}

.resource-url {
  @apply text-sm text-muted-foreground mb-2;
}

.resource-description {
  @apply text-sm mb-3;
}

.resource-tags {
  @apply flex flex-wrap gap-2;
}

.tag {
  @apply px-2 py-1 bg-secondary text-secondary-foreground rounded text-xs;
}
</style>
```

#### 2. 使用组件

```vue
<!-- src/views/ResourcesView.vue -->
<template>
  <div class="resources-view">
    <ResourceCard
      v-for="resource in resources"
      :key="resource.id"
      :resource="resource"
      @click="handleResourceClick"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import ResourceCard from '@/components/ResourceCard.vue'
import { apiService } from '@/services/api'
import type { Resource } from '@/types'

const resources = ref<Resource[]>([])

const handleResourceClick = (resource: Resource) => {
  console.log('Clicked resource:', resource.title)
}

onMounted(async () => {
  try {
    resources.value = await apiService.getResources()
  } catch (error) {
    console.error('Failed to load resources:', error)
  }
})
</script>
```

### 状态管理

#### Pinia Store

```typescript
// src/stores/resources.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { apiService } from '@/services/api'
import type { Resource, CreateResourceRequest } from '@/types'

export const useResourceStore = defineStore('resources', () => {
  // State
  const resources = ref<Resource[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Getters
  const favoriteResources = computed(() => 
    resources.value.filter(r => r.is_favorite)
  )
  
  const resourcesByCollection = computed(() => {
    const grouped: Record<number, Resource[]> = {}
    resources.value.forEach(resource => {
      const collectionId = resource.collection_id || 0
      if (!grouped[collectionId]) {
        grouped[collectionId] = []
      }
      grouped[collectionId].push(resource)
    })
    return grouped
  })

  const resourcesByType = computed(() => {
    const grouped: Record<string, Resource[]> = {}
    resources.value.forEach(resource => {
      const type = resource.resource_type
      if (!grouped[type]) {
        grouped[type] = []
      }
      grouped[type].push(resource)
    })
    return grouped
  })

  // Actions
  const fetchResources = async () => {
    loading.value = true
    error.value = null
    
    try {
      resources.value = await apiService.getResources()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch resources'
    } finally {
      loading.value = false
    }
  }

  const createResource = async (data: CreateResourceRequest) => {
    try {
      const newResource = await apiService.createResource(data)
      resources.value.unshift(newResource)
      return newResource
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to create resource'
      throw err
    }
  }

  const deleteResource = async (id: number) => {
    try {
      await apiService.deleteResource(id)
      resources.value = resources.value.filter(r => r.id !== id)
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to delete resource'
      throw err
    }
  }

  return {
    // State
    resources,
    loading,
    error,
    
    // Getters
    favoriteResources,
    resourcesByCollection,
    resourcesByType,
    
    // Actions
    fetchResources,
    createResource,
    deleteResource
  }
})
```

## 🗄️ 数据库开发

### 创建迁移

```sql
-- migrations/20231201000007_add_resource_metadata.sql
-- 添加资源类型和元数据字段
ALTER TABLE resources ADD COLUMN resource_type TEXT NOT NULL DEFAULT 'link';
ALTER TABLE resources ADD COLUMN file_path TEXT;
ALTER TABLE resources ADD COLUMN file_size INTEGER;
ALTER TABLE resources ADD COLUMN file_type TEXT;
ALTER TABLE resources ADD COLUMN metadata TEXT DEFAULT '{}';

-- 创建索引
CREATE INDEX idx_resources_metadata ON resources(metadata);
CREATE INDEX idx_resources_type ON resources(resource_type);
```

### 数据库模型

```rust
// src/models/resource.rs
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Resource {
    pub id: i64,
    pub title: String,
    pub url: Option<String>,
    pub description: Option<String>,
    pub resource_type: String, // 'link', 'file', 'note'
    pub user_id: i64,
    pub collection_id: Option<i64>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub file_type: Option<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub is_read: bool,
    pub visit_count: i32,
    pub last_visited: Option<chrono::DateTime<chrono::Utc>>,
    pub reading_time: Option<i32>,
    pub difficulty_level: Option<i32>,
    pub metadata: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateResourceRequest {
    pub title: String,
    pub url: Option<String>,
    pub description: Option<String>,
    pub resource_type: String,
    pub collection_id: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub reading_time: Option<i32>,
    pub difficulty_level: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub file_path: Option<String>,
    pub file_content: Option<String>, // for notes
}
```

### 数据库服务

```rust
// src/services/resource_service.rs
use sqlx::SqlitePool;
use crate::models::resource::{Resource, CreateResourceRequest};
use crate::utils::error::AppError;

pub struct ResourceService;

impl ResourceService {
    pub async fn create_resource(
        user_id: i64,
        request: CreateResourceRequest,
        db_pool: &SqlitePool,
    ) -> Result<Resource, AppError> {
        let resource = sqlx::query_as!(
            Resource,
            r#"
            INSERT INTO resources (
                title, url, description, resource_type, user_id, collection_id,
                file_path, file_size, file_type, is_favorite, reading_time, 
                difficulty_level, metadata
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
            request.title,
            request.url,
            request.description,
            request.resource_type,
            user_id,
            request.collection_id,
            request.file_path,
            None, // file_size calculated from file_path
            None, // file_type determined from file_path
            request.is_favorite.unwrap_or(false),
            request.reading_time,
            request.difficulty_level,
            request.metadata.map(|v| v.to_string())
        )
        .fetch_one(db_pool)
        .await?;

        Ok(resource)
    }

    pub async fn get_user_resources(
        user_id: i64,
        resource_type: Option<String>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Resource>, AppError> {
        let mut query = "
            SELECT * FROM resources 
            WHERE user_id = ?".to_string();
        
        let mut params = vec![user_id];
        
        if let Some(rt) = resource_type {
            query.push_str(" AND resource_type = ?");
            params.push(rt.parse().unwrap_or(0));
        }
        
        query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
        
        let bookmarks = sqlx::query_as!(
            Resource,
            &query,
            params[0],
            limit.unwrap_or(20),
            offset.unwrap_or(0)
        )
        .fetch_all(db_pool)
        .await?;

        Ok(bookmarks)
    }
}
```

## 🐛 调试技巧

### 后端调试

#### 日志配置

```rust
// src/main.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 配置日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 应用代码
}
```

#### 调试宏

```rust
use tracing::{debug, error, info, warn};

pub async fn process_resource(resource: &Resource) -> Result<(), AppError> {
    debug!("Processing resource: {} ({})", resource.title, resource.resource_type);
    
    match resource.resource_type.as_str() {
        "link" => {
            if resource.url.as_ref().map_or(true, |u| u.is_empty()) {
                warn!("Link resource has empty URL: {}", resource.id);
                return Err(AppError::InvalidInput("URL cannot be empty for link resources".to_string()));
            }
        },
        "file" => {
            if resource.file_path.is_none() {
                warn!("File resource has no file path: {}", resource.id);
                return Err(AppError::InvalidInput("File path is required for file resources".to_string()));
            }
        },
        "note" => {
            // 处理笔记内容
        },
        _ => {
            warn!("Unknown resource type: {}", resource.resource_type);
        }
    }
    
    // 处理逻辑
    info!("Successfully processed resource: {}", resource.id);
    Ok(())
}
```

#### 数据库调试

```bash
# 启用 SQLx 日志
RUST_LOG=sqlx=debug cargo run

# 查看数据库查询
export RUST_LOG=debug
cargo run
```

### 前端调试

#### Vue DevTools

安装 Vue DevTools 浏览器扩展进行调试。

#### 控制台调试

```typescript
// src/utils/debug.ts
export const debug = {
  log: (...args: any[]) => {
    if (import.meta.env.DEV) {
      console.log('[DEBUG]', ...args)
    }
  },
  
  error: (...args: any[]) => {
    console.error('[ERROR]', ...args)
  },
  
  group: (label: string, fn: () => void) => {
    if (import.meta.env.DEV) {
      console.group(label)
      fn()
      console.groupEnd()
    }
  }
}
```

#### 网络请求调试

```typescript
// src/services/api.ts
class ApiService {
  private async request<T>(endpoint: string, options: RequestInit = {}) {
    const url = `${API_BASE_URL}${endpoint}`
    
    if (import.meta.env.DEV) {
      console.log(`[API] ${options.method || 'GET'} ${url}`, {
        headers: options.headers,
        body: options.body
      })
    }
    
    try {
      const response = await fetch(url, options)
      
      if (import.meta.env.DEV) {
        console.log(`[API] Response ${response.status}`, response)
      }
      
      return await response.json()
    } catch (error) {
      console.error(`[API] Error`, error)
      throw error
    }
  }
}
```

## ⚡ 性能优化

### 后端优化

#### 数据库查询优化

```rust
// 使用索引
CREATE INDEX idx_resources_user_created ON resources(user_id, created_at DESC);

// 分页查询
pub async fn get_resources_paginated(
    user_id: i64,
    resource_type: Option<String>,
    page: i64,
    limit: i64,
) -> Result<Vec<Resource>, AppError> {
    let offset = (page - 1) * limit;
    
    let mut query = "SELECT * FROM resources WHERE user_id = ?".to_string();
    let mut params = vec![user_id];
    
    if let Some(rt) = resource_type {
        query.push_str(" AND resource_type = ?");
        params.push(rt.parse().unwrap_or(0));
    }
    
    query.push_str(" ORDER BY created_at DESC LIMIT ? OFFSET ?");
    
    let resources = sqlx::query_as!(
        Resource,
        &query,
        params[0],
        limit,
        offset
    )
    .fetch_all(db_pool)
    .await?;

    Ok(resources)
}

// 批量操作
pub async fn get_resources_with_tags(
    user_id: i64,
) -> Result<Vec<ResourceWithTags>, AppError> {
    let resources = sqlx::query_as!(
        ResourceWithTags,
        r#"
        SELECT 
            r.*,
            GROUP_CONCAT(t.name, ',') as tags
        FROM resources r
        LEFT JOIN resource_tags rt ON r.id = rt.resource_id
        LEFT JOIN tags t ON rt.tag_id = t.id
        WHERE r.user_id = ?
        GROUP BY r.id
        ORDER BY r.created_at DESC
        "#,
        user_id
    )
    .fetch_all(db_pool)
    .await?;

    Ok(resources)
}
```

#### 连接池配置

```rust
// src/config/database.rs
use sqlx::{sqlite::SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;

pub async fn create_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let connect_options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(30));

    let pool = SqlitePool::connect_with(connect_options).await?;
    
    // 优化连接池
    Ok(pool)
}
```

### 前端优化

#### 组件懒加载

```typescript
// src/router/index.ts
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('@/views/HomeView.vue')
    },
    {
      path: '/resources',
      name: 'resources',
      component: () => import('@/views/ResourcesView.vue')
    }
  ]
})
```

#### 虚拟滚动

```vue
<!-- src/components/VirtualList.vue -->
<template>
  <div class="virtual-list" :style="{ height: containerHeight + 'px' }">
    <div :style="{ height: totalHeight + 'px', position: 'relative' }">
      <div
        v-for="item in visibleItems"
        :key="item.id"
        :style="{
          position: 'absolute',
          top: item.top + 'px',
          width: '100%'
        }"
      >
        <slot :item="item.data" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'

interface Props {
  items: any[]
  itemHeight: number
  containerHeight: number
}

const props = defineProps<Props>()

const scrollTop = ref(0)

const totalHeight = computed(() => props.items.length * props.itemHeight)

const visibleItems = computed(() => {
  const start = Math.floor(scrollTop.value / props.itemHeight)
  const end = Math.min(
    start + Math.ceil(props.containerHeight / props.itemHeight) + 1,
    props.items.length
  )
  
  return props.items.slice(start, end).map((item, index) => ({
    id: item.id,
    data: item,
    top: (start + index) * props.itemHeight
  }))
})

const handleScroll = (event: Event) => {
  scrollTop.value = (event.target as HTMLElement).scrollTop
}

onMounted(() => {
  const container = document.querySelector('.virtual-list')
  container?.addEventListener('scroll', handleScroll)
})

onUnmounted(() => {
  const container = document.querySelector('.virtual-list')
  container?.removeEventListener('scroll', handleScroll)
})
</script>
```

## 🤝 贡献指南

### 开始贡献

1. **Fork 项目**
2. **创建功能分支**
3. **编写代码和测试**
4. **提交 Pull Request**

### 代码贡献规范

- 遵循项目的代码规范
- 编写清晰的提交信息
- 添加必要的测试
- 更新相关文档

### 问题报告

使用 GitHub Issues 报告问题时，请包含：

- 问题描述
- 复现步骤
- 期望行为
- 实际行为
- 环境信息

### 文档贡献

- 修复文档错误
- 改进示例代码
- 添加使用指南
- 翻译文档

---

**更新时间**: 2025-12-02
**版本**: 1.0.0
