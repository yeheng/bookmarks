# API 接口规范文档

## 概述

本文档定义了多资源聚合系统的 RESTful API 接口规范，包括请求格式、响应格式、错误处理和认证机制。API 遵循 REST 设计原则，使用 JSON 格式进行数据交换，支持链接、文件、笔记等多种类型资源的统一管理。该 API 专为 Vue.js 3 + Reka UI 前端架构设计，提供高效的数据交互和良好的开发体验。

## 基础信息

- **Base URL**: `http://localhost:3000/api`
- **API 版本**: v1
- **Content-Type**: `application/json`
- **字符编码**: UTF-8
- **认证方式**: Bearer Token (JWT)
- **前端框架**: Vue.js 3.4+
- **UI 组件库**: Reka UI (基于 Radix Vue)
- **状态管理**: Pinia
- **数据库**: SQLite with FTS5
- **后端框架**: Rust + Axum
- **搜索支持**: 中英文混合全文搜索

## 通用响应格式

### 成功响应

```json
{
  "success": true,
  "data": {
    // 响应数据
  },
  "message": "操作成功"
}
```

### 分页响应

```json
{
  "success": true,
  "data": {
    "items": [
      // 数据项列表
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 100,
      "total_pages": 5,
      "has_next": true,
      "has_prev": false
    }
  },
  "message": "获取成功",
  "search_time": 0.05
}
```

### 错误响应

```json
{
  "success": false,
  "message": "错误描述",
  "code": "ERROR_CODE",
  "details": {
    // 详细错误信息
  }
}
```

注意：当前版本使用标准分页，通过查询参数 `limit` 和 `offset` 控制，返回完整的分页信息。

## 认证接口

### 1. 用户注册

**POST** `/auth/register`

注册新用户账户。

**请求体**:

```json
{
  "username": "testuser",
  "email": "test@example.com",
  "password": "password123"
}
```

**请求参数**:

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| username | string | 是 | 用户名，3-50个字符 |
| email | string | 是 | 邮箱地址 |
| password | string | 是 | 密码，最少8个字符 |

**响应**:

```json
{
  "success": true,
  "data": {
    "user": {
      "id": 1,
      "username": "testuser",
      "email": "test@example.com",
      "created_at": 1735584000
    },
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  },
  "message": "注册成功"
}
```

**错误码**:

- `INVALID_INPUT`: 输入数据无效
- `USER_EXISTS`: 用户名或邮箱已存在
- `WEAK_PASSWORD`: 密码强度不足

### 2. 用户登录

**POST** `/auth/login`

用户登录获取访问令牌。

**请求体**:

```json
{
  "email": "test@example.com",
  "password": "password123"
}
```

**请求参数**:

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| email | string | 是 | 邮箱地址 |
| password | string | 是 | 密码 |

**响应**:

```json
{
  "success": true,
  "data": {
    "user": {
      "id": 1,
      "username": "testuser",
      "email": "test@example.com",
      "last_login_at": 1735584000
    },
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  },
  "message": "登录成功"
}
```

**错误码**:

- `INVALID_CREDENTIALS`: 邮箱或密码错误
- `USER_INACTIVE`: 用户账户已被禁用
- `RATE_LIMIT_EXCEEDED`: 请求频率过高

### 3. 刷新令牌

**POST** `/auth/refresh`

使用刷新令牌获取新的访问令牌。

**请求体**:

```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**请求参数**:

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| refresh_token | string | 是 | 刷新令牌 |

**响应**:

```json
{
  "success": true,
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expires_in": 900
  },
  "message": "令牌刷新成功"
}
```

**错误码**:

- `INVALID_TOKEN`: 刷新令牌无效或已过期
- `TOKEN_REVOKED`: 刷新令牌已被撤销

### 4. 用户登出

**POST** `/auth/logout`

用户登出，使当前令牌失效。

**请求头**:

```
Authorization: Bearer <access_token>
```

**响应**:

```json
{
  "success": true,
  "data": null,
  "message": "登出成功"
}
```

### 5. 获取当前用户信息

**GET** `/auth/me`

获取当前认证用户的详细信息。

**请求头**:

```
Authorization: Bearer <access_token>
```

**响应**:

```json
{
  "success": true,
  "data": {
    "id": 1,
    "username": "testuser",
    "email": "test@example.com",
    "avatar_url": "https://ui-avatars.com/api/?name=testuser&background=3b82f6&color=fff",
    "is_active": true,
    "email_verified": true,
    "last_login_at": 1735584000,
    "created_at": 1735584000,
    "updated_at": 1735584000
  },
  "message": "获取成功"
}
```

## 资源接口

### 1. 获取资源列表

**GET** `/resources`

获取用户的资源列表，支持分页和过滤。资源包括链接、文件、笔记等多种类型。

**请求头**:

```
Authorization: Bearer <access_token>
```

**查询参数**:

| 参数 | 类型 | 必需 | 默认值 | 描述 |
|------|------|------|--------|------|
| limit | integer | 否 | 20 | 每页数量，最大100 |
| offset | integer | 否 | 0 | 偏移量 |
| collection_id | number | 否 | - | 收藏夹ID |
| tag_id | number | 否 | - | 标签ID |
| resource_type | string | 否 | - | 资源类型 (link/file/note) |
| q | string | 否 | - | 搜索关键词 |

**响应**:

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": 1,
        "title": "示例网站",
        "url": "https://example.com",
        "description": "这是一个示例网站",
        "resource_type": "link",
        "user_id": 1,
        "collection_id": 1,
        "collection_name": "技术文档",
        "collection_color": "#3b82f6",
        "created_at": 1735584000,
        "updated_at": 1735584000,
        "tags": ["技术", "前端"],
        "is_archived": false,
        "is_favorite": true,
        "is_private": false,
        "is_read": false,
        "visit_count": 5,
        "last_visited": 1735584000,
        "metadata": {},
        "reading_time": 3,
        "difficulty_level": 2,
        "favicon_url": "https://example.com/favicon.ico",
        "file_size": null,
        "file_type": null
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 1,
      "total_pages": 1,
      "has_next": false,
      "has_prev": false
    }
  },
  "message": "获取成功"
}
```

### 2. 获取单个资源

**GET** `/resources/{id}`

获取指定资源的详细信息。

**请求头**:

```
Authorization: Bearer <access_token>
```

**路径参数**:

| 参数 | 类型 | 描述 |
|------|------|------|
| id | string | 资源ID |

**响应**:

```json
{
  "success": true,
  "data": {
    "id": 1,
    "title": "示例网站",
    "url": "https://example.com",
    "description": "这是一个示例网站",
    "favicon_url": "https://example.com/favicon.ico",
    "screenshot_url": "https://example.com/screenshot.png",
    "thumbnail_url": "https://example.com/thumbnail.png",
    "is_favorite": true,
    "is_archived": false,
    "is_read": false,
    "is_private": false,
    "visit_count": 5,
    "last_visited": 1735584000,
    "reading_time": 3,
    "difficulty_level": 2,
    "metadata": {
      "author": "张三",
      "publish_date": "2025-11-29"
    },
    "tags": ["技术", "前端"],
    "collection_name": "技术文档",
    "collection_color": "#3b82f6",
    "user_id": 1,
    "collection_id": 1,
    "created_at": 1735584000,
    "updated_at": 1735584000
  },
  "message": "获取成功"
}
```

**错误码**:

- `RESOURCE_NOT_FOUND`: 资源不存在
- `ACCESS_DENIED`: 无权访问该书签

### 3. 创建书签

**POST** `/bookmarks`

创建新的书签。

**请求头**:

```
Authorization: Bearer <access_token>
```

**请求体**:

```json
{
  "title": "示例网站",
  "url": "https://example.com",
  "description": "这是一个示例网站",
  "collection_id": "550e8400-e29b-41d4-a716-446655440001",
  "tags": ["技术", "前端"],
  "is_favorite": true,
  "is_private": false,
  "reading_time": 3,
  "difficulty_level": 2,
  "metadata": {
    "author": "张三",
    "publish_date": "2025-11-29"
  }
}
```

**请求参数**:

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| title | string | 是 | 书签标题 |
| url | string | 是 | 书签URL |
| description | string | 否 | 书签描述 |
| collection_id | string | 否 | 收藏夹ID |
| tags | string[] | 否 | 标签列表 |
| is_favorite | boolean | 否 | 是否收藏 |
| is_private | boolean | 否 | 是否私有 |
| reading_time | integer | 否 | 预估阅读时间（分钟） |
| difficulty_level | integer | 否 | 难度等级（1-5） |
| metadata | object | 否 | 额外元数据 |

**响应**:

```json
{
  "success": true,
  "data": {
    "id": 1,
    "title": "示例网站",
    "url": "https://example.com",
    "description": "这是一个示例网站",
    "is_favorite": true,
    "is_private": false,
    "tags": ["技术", "前端"],
    "collection_name": "技术文档",
    "collection_color": "#3b82f6",
    "user_id": 1,
    "collection_id": 1,
    "created_at": 1735584000,
    "updated_at": 1735584000,
    "visit_count": 0,
    "is_archived": false,
    "is_read": false,
    "metadata": {}
  },
  "message": "创建成功"
}
```

**错误码**:

- `INVALID_URL`: URL格式无效
- `RESOURCE_EXISTS`: 资源已存在
- `COLLECTION_NOT_FOUND`: 收藏夹不存在

### 4. 更新书签

**PUT** `/bookmarks/{id}`

更新指定书签的信息。

**请求头**:

```
Authorization: Bearer <access_token>
```

**路径参数**:

| 参数 | 类型 | 描述 |
|------|------|------|
| id | string | 资源ID |

**请求体**:

```json
{
  "title": "更新后的标题",
  "description": "更新后的描述",
  "collection_id": null,
  "tags": ["技术", "后端"],
  "is_favorite": false,
  "is_archived": true
}
```

**请求参数**:

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| title | string | 否 | 书签标题 |
| url | string | 否 | 书签URL |
| description | string | 否 | 书签描述 |
| collection_id | string | 否 | 收藏夹ID，null表示移出收藏夹 |
| tags | string[] | 否 | 标签列表 |
| is_favorite | boolean | 否 | 是否收藏 |
| is_archived | boolean | 否 | 是否归档 |
| is_read | boolean | 否 | 是否已读 |
| is_private | boolean | 否 | 是否私有 |
| reading_time | integer | 否 | 预估阅读时间 |
| difficulty_level | integer | 否 | 难度等级 |
| metadata | object | 否 | 额外元数据 |

**响应**:

```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "更新后的标题",
    "description": "更新后的描述",
    "is_favorite": false,
    "is_archived": true,
    "tags": ["技术", "后端"],
    "collection": null,
    "updated_at": "2025-11-30T10:30:00Z"
  },
  "message": "更新成功"
}
```

### 5. 删除资源

**DELETE** `/resources/{id}`

删除指定的资源。

**请求头**:

```
Authorization: Bearer <access_token>
```

**路径参数**:

| 参数 | 类型 | 描述 |
|------|------|------|
| id | string | 资源ID |

**响应**:

```json
{
  "success": true,
  "data": null,
  "message": "删除成功"
}
```

### 6. 记录访问

**POST** `/resources/{id}/visit`

记录资源访问，增加访问次数。

**请求头**:

```
Authorization: Bearer <access_token>
```

**路径参数**:

| 参数 | 类型 | 描述 |
|------|------|------|
| id | string | 资源ID |

**响应**:

```json
{
  "success": true,
  "data": {
    "visit_count": 6,
    "last_visited": "2025-11-30T10:00:00Z"
  },
  "message": "访问记录成功"
}
```

### 7. 批量操作

**POST** `/bookmarks/batch`

批量操作书签。

**请求头**:

```
Authorization: Bearer <access_token>
```

**请求体**:

```json
{
  "action": "delete",
  "bookmark_ids": [
    1,
    2
  ],
  "data": {
    "collection_id": 3,
    "tags": ["新标签"]
  }
}
```

**请求参数**:

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| action | string | 是 | 操作类型 (delete/move/add_tags/remove_tags) |
| resource_ids | number[] | 是 | 资源ID列表 |
| data | object | 否 | 操作数据 |

**响应**:

```json
{
  "success": true,
  "data": {
    "processed": 2,
    "failed": 0,
    "errors": []
  },
  "message": "批量操作完成"
}
```

### 8. 导入资源

**POST** `/resources/import`

从文件导入资源。

**请求头**:

```
Authorization: Bearer <access_token>
Content-Type: multipart/form-data
```

**请求体**:

```
file: <bookmark_file>
format: json | html | netscape
collection_id: <collection_id> (可选)
```

**响应**:

```json
{
  "success": true,
  "data": {
    "imported": 25,
    "skipped": 3,
    "duplicates": 2,
    "errors": []
  },
  "message": "导入完成"
}
```

### 9. 导出资源

**GET** `/resources/export`

导出用户资源。

**请求头**:

```
Authorization: Bearer <access_token>
```

**查询参数**:

| 参数 | 类型 | 必需 | 默认值 | 描述 |
|------|------|------|--------|------|
| format | string | 否 | json | 导出格式 (json/html/netscape) |
| collection_id | string | 否 | - | 指定收藏夹 |
| include_archived | boolean | 否 | false | 是否包含归档书签 |

**响应**: 文件下载或JSON数据

## 收藏夹接口

### 1. 获取收藏夹列表

**GET** `/collections`

获取用户的收藏夹列表。

**请求头**:

```
Authorization: Bearer <access_token>
```

**查询参数**:

| 参数 | 类型 | 必需 | 默认值 | 描述 |
|------|------|------|--------|------|
| include_count | boolean | 否 | true | 是否包含书签数量 |

**响应**:

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": 1,
        "name": "技术文档",
        "description": "技术相关的文档和教程",
        "color": "#3b82f6",
        "icon": "folder",
        "sort_order": 0,
        "is_default": false,
        "is_public": false,
        "bookmark_count": 15,
        "parent_id": null,
        "user_id": 1,
        "created_at": 1735584000,
        "updated_at": 1735584000
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 1,
      "total_pages": 1,
      "has_next": false,
      "has_prev": false
    }
  },
  "message": "获取成功"
}
```

### 2. 创建收藏夹

**POST** `/collections`

创建新的收藏夹。

**请求头**:

```
Authorization: Bearer <access_token>
```

**请求体**:

```json
{
  "name": "技术文档",
  "description": "技术相关的文档和教程",
  "color": "#3b82f6",
  "icon": "folder",
  "parent_id": null
}
```

**请求参数**:

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| name | string | 是 | 收藏夹名称 |
| description | string | 否 | 收藏夹描述 |
| color | string | 否 | 颜色代码，默认#3b82f6 |
| icon | string | 否 | 图标名称，默认folder |
| parent_id | string | 否 | 父收藏夹ID |

**响应**:

```json
{
  "success": true,
  "data": {
    "id": 1,
    "name": "技术文档",
    "description": "技术相关的文档和教程",
    "color": "#3b82f6",
    "icon": "folder",
    "sort_order": 0,
    "is_default": false,
    "bookmark_count": 0,
    "parent_id": null,
    "user_id": 1,
    "created_at": 1735584000,
    "updated_at": 1735584000
  },
  "message": "创建成功"
}
```

### 3. 更新收藏夹

**PUT** `/collections/{id}`

更新收藏夹信息。

**请求头**:

```
Authorization: Bearer <access_token>
```

**请求体**:

```json
{
  "name": "更新后的名称",
  "description": "更新后的描述",
  "color": "#ef4444"
}
```

### 4. 删除收藏夹

**DELETE** `/collections/{id}`

删除收藏夹。可选择是否同时删除其中的书签。

**请求头**:

```
Authorization: Bearer <access_token>
```

**查询参数**:

| 参数 | 类型 | 必需 | 默认值 | 描述 |
|------|------|------|--------|------|
| move_bookmarks | boolean | 否 | true | 是否将书签移动到默认收藏夹 |

## 标签接口

### 1. 获取标签列表

**GET** `/tags`

获取用户的标签列表。

**请求头**:

```
Authorization: Bearer <access_token>
```

**查询参数**:

| 参数 | 类型 | 必需 | 默认值 | 描述 |
|------|------|------|--------|------|
| include_count | boolean | 否 | true | 是否包含使用次数 |
| sort | string | 否 | usage_count | 排序字段 (name/usage_count/created_at) |
| order | string | 否 | desc | 排序方向 |

**响应**:

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": 1,
        "name": "技术",
        "color": "#64748b",
        "description": "技术相关的标签",
        "usage_count": 25,
        "user_id": 1,
        "created_at": 1735584000,
        "updated_at": 1735584000
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 1,
      "total_pages": 1,
      "has_next": false,
      "has_prev": false
    }
  },
  "message": "获取成功"
}
```

### 2. 获取热门标签

**GET** `/tags/popular`

获取热门标签列表，按使用频率排序。

**请求头**:

```
Authorization: Bearer <access_token>
```

**响应**:

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": 1,
        "name": "技术",
        "color": "#64748b",
        "description": "技术相关的标签",
        "usage_count": 25,
        "user_id": 1,
        "created_at": 1735584000,
        "updated_at": 1735584000
      }
    ]
  },
  "message": "获取成功"
}
```

### 2. 创建标签

**POST** `/tags`

创建新标签。

**请求头**:

```
Authorization: Bearer <access_token>
```

**请求体**:

```json
{
  "name": "新技术",
  "color": "#10b981",
  "description": "新技术相关内容"
}
```

### 3. 更新标签

**PUT** `/tags/{id}`

更新标签信息。

### 4. 删除标签

**DELETE** `/tags/{id}`

删除标签。

## 搜索接口

### 1. 搜索资源

**GET** `/search/resources`

全文搜索资源，支持链接、文件、笔记等多种类型。

**请求头**:

```
Authorization: Bearer <access_token>
```

**查询参数**:

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| q | string | 否 | 搜索关键词（全局搜索） |
| search | string | 否 | 搜索关键词（特定搜索） |
| collection_id | number | 否 | 限制在指定收藏夹中搜索 |
| tags | string | 否 | 限制在指定标签中搜索（逗号分隔） |
| is_favorite | boolean | 否 | 是否收藏 |
| is_archived | boolean | 否 | 是否归档 |
| is_private | boolean | 否 | 是否私有 |
| is_read | boolean | 否 | 是否已读 |
| limit | number | 否 | 每页数量，默认20 |
| offset | number | 否 | 偏移量，默认0 |
| sort_by | string | 否 | 排序字段 (created_at/updated_at/title/visit_count) |
| sort_order | string | 否 | 排序方向 (asc/desc) |

**响应**:

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": 1,
        "title": "示例网站",
        "url": "https://example.com",
        "description": "这是一个示例网站",
        "tags": ["技术"],
        "collection_name": "技术文档",
        "collection_color": "#3b82f6",
        "user_id": 1,
        "collection_id": 1,
        "created_at": 1735584000,
        "updated_at": 1735584000,
        "is_archived": false,
        "is_favorite": true,
        "is_private": false,
        "is_read": false,
        "visit_count": 5,
        "last_visited": 1735584000,
        "metadata": {},
        "reading_time": 3,
        "difficulty_level": 2
      }
    ],
    "pagination": {
      "page": 1,
      "limit": 20,
      "total": 1,
      "total_pages": 1,
      "has_next": false,
      "has_prev": false
    },
    "highlights": {
      "title": ["<mark>示例</mark>网站"],
      "description": ["这是一个<mark>示例</mark>网站"]
    }
  },
  "message": "搜索完成",
  "search_time": 0.05
}
```

### 2. 搜索建议

**GET** `/search/suggestions`

获取搜索建议。

**请求头**:

```
Authorization: Bearer <access_token>
```

**查询参数**:

| 参数 | 类型 | 必需 | 描述 |
|------|------|------|------|
| q | string | 是 | 搜索关键词前缀 |
| limit | integer | 否 | 建议数量，默认10 |

**响应**:

```json
{
  "success": true,
  "data": {
    "suggestions": [
      {
        "text": "JavaScript 教程",
        "type": "bookmark",
        "count": 5
      },
      {
        "text": "JavaScript",
        "type": "tag",
        "count": 15
      }
    ]
  },
  "message": "获取成功"
}
```

## 统计接口

### 1. 用户统计

**GET** `/stats/user`

获取用户统计数据。

**请求头**:

```
Authorization: Bearer <access_token>
```

**查询参数**:

无

**响应**:

```json
{
  "success": true,
  "data": {
    "total_resources": 150,
    "total_collections": 8,
    "total_tags": 25,
    "favorite_resources": 20,
    "archived_resources": 10,
    "total_visits": 1250,
    "recent_resources": [
      {
        "id": 1,
        "title": "最近添加的书签",
        "url": "https://example.com",
        "created_at": 1735584000
      }
    ],
    "recent_activity": [
      {
        "date": 1735584000,
        "resources_added": 3,
        "resources_visited": 12
      }
    ],
    "top_tags": [
      {
        "name": "技术",
        "count": 45
      }
    ],
    "top_domains": [
      {
        "domain": "github.com",
        "count": 25
      }
    ]
  },
  "message": "获取成功"
}
```

## 错误码说明

| 错误码 | HTTP状态码 | 描述 |
|--------|------------|------|
| `INVALID_REQUEST` | 400 | 请求格式错误 |
| `INVALID_INPUT` | 400 | 输入数据无效 |
| `INVALID_URL` | 400 | URL格式无效 |
| `MISSING_REQUIRED_FIELD` | 400 | 缺少必需字段 |
| `UNAUTHORIZED` | 401 | 未认证或认证失败 |
| `INVALID_TOKEN` | 401 | 令牌无效或已过期 |
| `TOKEN_REVOKED` | 401 | 令牌已被撤销 |
| `ACCESS_DENIED` | 403 | 权限不足 |
| `RESOURCE_NOT_FOUND` | 404 | 资源不存在 |
| `RESOURCE_NOT_FOUND` | 404 | 资源不存在 |
| `COLLECTION_NOT_FOUND` | 404 | 收藏夹不存在 |
| `TAG_NOT_FOUND` | 404 | 标签不存在 |
| `USER_NOT_FOUND` | 404 | 用户不存在 |
| `RESOURCE_EXISTS` | 409 | 资源已存在 |
| `USER_EXISTS` | 409 | 用户已存在 |
| `RESOURCE_EXISTS` | 409 | 资源已存在 |
| `TAG_EXISTS` | 409 | 标签已存在 |
| `RATE_LIMIT_EXCEEDED` | 429 | 请求频率过高 |
| `INTERNAL_ERROR` | 500 | 内部服务器错误 |
| `DATABASE_ERROR` | 500 | 数据库错误 |
| `EXTERNAL_SERVICE_ERROR` | 502 | 外部服务错误 |
| `SERVICE_UNAVAILABLE` | 503 | 服务不可用 |

## 请求限制

### 频率限制

- 认证接口：每分钟最多 10 次请求
- 搜索接口：每分钟最多 30 次请求
- 其他接口：每分钟最多 100 次请求

### 数据限制

- 书签标题：最大 255 字符
- 书签描述：最大 1000 字符
- 收藏夹名称：最大 100 字符
- 标签名称：最大 50 字符
- 文件上传：最大 10MB

## 版本控制

API 版本通过 URL 路径进行管理：

- v1: `/api/v1/...`
- v2: `/api/v2/...` (未来版本)

向后兼容性保证：

- 新增字段不影响现有客户端
- 废弃字段会提前通知
- 重大变更会发布新版本

## 开发工具

### Postman 集合

提供完整的 Postman 集合文件，包含所有 API 接口的示例请求。

### OpenAPI 规范

API 同时提供 OpenAPI 3.0 规范文件，可用于：

- 自动生成客户端 SDK
- API 文档生成
- 接口测试

### 示例代码

提供多种语言的示例代码：

- JavaScript/TypeScript
- Python
- Rust
- Go

---

这个 API 接口规范为多资源聚合系统提供了完整、标准化的接口定义，支持链接、文件、笔记等多种类型资源的管理，并考虑了安全性、性能和可扩展性。
