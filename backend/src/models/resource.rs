use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row};

/// 资源类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResourceType {
    Link,    // 链接型资源
    Note,    // 笔记型资源
    Snippet, // 代码片段
    File,    // 文件资源
}

impl ResourceType {
    /// 从字符串解析资源类型
    pub fn from(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "link" => Ok(ResourceType::Link),
            "note" => Ok(ResourceType::Note),
            "snippet" => Ok(ResourceType::Snippet),
            "file" => Ok(ResourceType::File),
            _ => Err(format!("Unknown resource type: {}", s)),
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            ResourceType::Link => "link",
            ResourceType::Note => "note",
            ResourceType::Snippet => "snippet",
            ResourceType::File => "file",
        }
    }
}

/// 资源主模型
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Resource {
    pub id: i64,
    pub user_id: i64,
    pub collection_id: Option<i64>,
    pub title: String,
    pub url: Option<String>, // 改为 Option,支持 note 等无 URL 类型
    pub description: Option<String>,
    pub favicon_url: Option<String>,
    pub screenshot_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub is_private: bool,
    pub is_read: bool,
    pub visit_count: i64,
    pub last_visited: Option<i64>,
    pub reading_time: Option<i32>,
    pub difficulty_level: Option<i32>,
    pub metadata: serde_json::Value,

    // 新增字段
    #[sqlx(rename = "type")]
    pub resource_type: String, // 数据库中的 type 字段
    pub content: Option<String>,   // 笔记/代码片段内容
    pub source: Option<String>,    // 文件来源
    pub mime_type: Option<String>, // MIME 类型

    pub created_at: i64,
    pub updated_at: i64,
}

impl Resource {
    /// 获取解析后的资源类型
    #[allow(unused)]
    pub fn get_type(&self) -> Result<ResourceType, String> {
        ResourceType::from(&self.resource_type)
    }
}

/// 创建资源请求
#[derive(Debug, Deserialize)]
pub struct CreateResource {
    pub title: String,
    pub url: Option<String>, // Link 必须,Note/Snippet/File 可选
    pub description: Option<String>,
    pub collection_id: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_private: Option<bool>,

    // 新增字段
    #[serde(rename = "type")]
    pub resource_type: String, // "link" | "note" | "snippet" | "file"
    pub content: Option<String>,   // Note/Snippet 内容
    pub source: Option<String>,    // File 来源
    pub mime_type: Option<String>, // File MIME 类型
}

/// 更新资源请求
#[derive(Debug, Deserialize)]
pub struct UpdateResource {
    pub title: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub collection_id: Option<i64>,
    pub clear_collection_id: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
    pub is_private: Option<bool>,
    pub is_read: Option<bool>,
    pub reading_time: Option<i32>,
    pub difficulty_level: Option<i32>,

    // 新增字段
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    pub content: Option<String>,
    pub source: Option<String>,
    pub mime_type: Option<String>,
}

/// 资源及其关联数据(标签、收藏夹)
#[derive(Debug, Clone, Serialize)]
pub struct ResourceWithTags {
    #[serde(flatten)]
    pub resource: Resource,
    pub tags: Vec<String>,
    pub collection_name: Option<String>,
    pub collection_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_count: Option<i64>, // 引用数量统计
}

// SQLite compatible FromRow implementation
impl<'r> FromRow<'r, sqlx::sqlite::SqliteRow> for ResourceWithTags {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        // 解析标签 JSON
        let tags_json: Option<String> = row.try_get("tags")?;
        let tags: Vec<String> = if let Some(tags_json) = tags_json {
            serde_json::from_str(&tags_json).unwrap_or_default()
        } else {
            Vec::new()
        };

        // 尝试获取引用计数(可能不存在)
        let reference_count: Option<i64> = row.try_get("reference_count").ok();

        Ok(ResourceWithTags {
            resource: Resource {
                id: row.try_get("id")?,
                user_id: row.try_get("user_id")?,
                collection_id: row.try_get("collection_id")?,
                title: row.try_get("title")?,
                url: row.try_get("url")?,
                description: row.try_get("description")?,
                favicon_url: row.try_get("favicon_url")?,
                screenshot_url: row.try_get("screenshot_url")?,
                thumbnail_url: row.try_get("thumbnail_url")?,
                is_favorite: row.try_get("is_favorite")?,
                is_archived: row.try_get("is_archived")?,
                is_private: row.try_get("is_private")?,
                is_read: row.try_get("is_read")?,
                visit_count: row.try_get("visit_count")?,
                last_visited: row.try_get("last_visited")?,
                reading_time: row.try_get("reading_time")?,
                difficulty_level: row.try_get("difficulty_level")?,
                metadata: row.try_get("metadata")?,
                resource_type: row.try_get("type")?,
                content: row.try_get("content")?,
                source: row.try_get("source")?,
                mime_type: row.try_get("mime_type")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
            },
            tags,
            collection_name: row.try_get("collection_name")?,
            collection_color: row.try_get("collection_color")?,
            reference_count,
        })
    }
}

/// 资源查询参数
#[derive(Debug, Deserialize, Default)]
pub struct ResourceQuery {
    pub collection_id: Option<i64>,
    pub tags: Option<Vec<String>>,
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
    pub is_private: Option<bool>,
    pub is_read: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<String>, // "created_at", "updated_at", "title", "visit_count"
    pub sort_order: Option<String>, // "asc", "desc"

    // 新增: 资源类型过滤
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
}

/// 批量操作动作
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceBatchAction {
    Delete,
    Move,
    AddTags,
    RemoveTags,
}

/// 批量操作数据
#[derive(Debug, Deserialize)]
pub struct ResourceBatchData {
    pub collection_id: Option<i64>,
    pub tags: Option<Vec<String>>,
}

/// 批量操作请求
#[derive(Debug, Deserialize)]
pub struct ResourceBatchRequest {
    pub action: ResourceBatchAction,
    pub resource_ids: Vec<i64>,
    pub data: Option<ResourceBatchData>,
}

/// 批量操作结果
#[derive(Debug, Serialize)]
pub struct ResourceBatchResult {
    pub processed: usize,
    pub failed: usize,
    pub errors: Vec<ResourceBatchError>,
}

/// 批量操作错误
#[derive(Debug, Serialize)]
pub struct ResourceBatchError {
    pub resource_id: i64,
    pub reason: String,
}

// ============================================================
// 资源引用模型
// ============================================================

/// 资源引用关系
#[allow(unused)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ResourceReference {
    pub id: i64,
    pub source_id: i64,
    pub target_id: i64,
    #[serde(rename = "type")]
    pub reference_type: String, // "related" | "depends_on" | "references"
    pub created_at: i64,
}

/// 创建资源引用
#[derive(Debug, Deserialize)]
pub struct CreateResourceReference {
    pub target_id: i64,
    #[serde(rename = "type")]
    pub reference_type: Option<String>, // 默认 "related"
}

/// 资源引用查询参数
#[derive(Debug, Deserialize)]
pub struct ResourceReferenceQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    #[serde(rename = "type")]
    pub reference_type: Option<String>, // 过滤引用类型
    pub direction: Option<String>, // "source" | "target" | "both"
}

/// 资源引用列表响应
#[derive(Debug, Serialize)]
pub struct ResourceReferenceList {
    pub items: Vec<ResourceWithTags>,
    pub limit: i64,
    pub offset: i64,
    pub has_more: bool,
}
