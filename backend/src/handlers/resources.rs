use axum::{
    extract::{Json, Path, Query, State},
    response::Response,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::middleware::AuthenticatedUser;
use crate::models::{
    CreateResource, CreateResourceReference, ResourceBatchRequest, ResourceBatchResult,
    ResourceQuery, ResourceReferenceQuery, UpdateResource,
};
use crate::services::ResourceService;
use crate::state::AppState;
use crate::utils::error::AppError;
use crate::utils::response::{
    success_message_response, success_response, success_response_with_message,
};

/// 资源列表查询参数
#[derive(Deserialize)]
pub struct ResourceListQuery {
    pub collection_id: Option<i64>,
    pub tags: Option<String>, // 逗号分隔
    pub is_favorite: Option<bool>,
    pub is_archived: Option<bool>,
    pub is_private: Option<bool>,
    pub is_read: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// 获取资源列表
/// 支持多种过滤条件和排序选项
pub async fn get_resources(
    State(db_pool): State<SqlitePool>,
    Query(query): Query<ResourceListQuery>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    // 解析逗号分隔的标签字符串
    let tags: Vec<String> = query
        .tags
        .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let resource_query = ResourceQuery {
        collection_id: query.collection_id,
        tags: if tags.is_empty() { None } else { Some(tags) },
        is_favorite: query.is_favorite,
        is_archived: query.is_archived,
        is_private: query.is_private,
        is_read: query.is_read,
        search: query.search,
        limit: query.limit,
        offset: query.offset,
        sort_by: query.sort_by,
        sort_order: query.sort_order,
        resource_type: None, // 暂不从查询参数中获取,后续可以扩展
    };

    let resources = ResourceService::get_resources(user_id, resource_query, &db_pool).await?;

    Ok(success_response(resources))
}

/// 根据ID获取单个资源
pub async fn get_resource(
    State(db_pool): State<SqlitePool>,
    Path(resource_id): Path<i64>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let resource = ResourceService::get_resource_by_id(user_id, resource_id, &db_pool).await?;

    Ok(success_response(resource))
}

/// 创建新资源
pub async fn create_resource(
    State(app_state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(resource_data): Json<CreateResource>,
) -> Result<Response, AppError> {
    let resource =
        ResourceService::create_resource(user_id, resource_data, &app_state.db_pool).await?;

    Ok(success_response(resource))
}

/// 更新资源信息
pub async fn update_resource(
    State(app_state): State<AppState>,
    Path(resource_id): Path<i64>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(update_data): Json<UpdateResource>,
) -> Result<Response, AppError> {
    let resource =
        ResourceService::update_resource(user_id, resource_id, update_data, &app_state.db_pool)
            .await?;

    Ok(success_response(resource))
}

/// 删除资源
pub async fn delete_resource(
    State(app_state): State<AppState>,
    Path(resource_id): Path<i64>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let deleted =
        ResourceService::delete_resource(user_id, resource_id, &app_state.db_pool).await?;

    if !deleted {
        return Err(AppError::NotFound("Resource not found".to_string()));
    }

    Ok(success_message_response("Resource deleted successfully"))
}

/// 批量更新资源
/// 支持批量修改标签、收藏夹状态等属性
pub async fn batch_update_resources(
    State(db_pool): State<SqlitePool>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(payload): Json<ResourceBatchRequest>,
) -> Result<Response, AppError> {
    if payload.resource_ids.is_empty() {
        return Err(AppError::BadRequest(
            "resource_ids cannot be empty".to_string(),
        ));
    }

    let result: ResourceBatchResult =
        ResourceService::batch_process(user_id, payload, &db_pool).await?;

    Ok(success_response_with_message(
        result,
        "Batch operation completed",
    ))
}

// ==================== 资源引用管理 ====================

/// 创建资源引用关系
/// 建立两个资源之间的关联关系
pub async fn create_resource_reference(
    State(db_pool): State<SqlitePool>,
    Path(resource_id): Path<i64>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(reference_data): Json<CreateResourceReference>,
) -> Result<Response, AppError> {
    // 验证源资源的所有权
    let _source = ResourceService::get_resource_by_id(user_id, resource_id, &db_pool).await?;

    // 验证目标资源的所有权
    let _target =
        ResourceService::get_resource_by_id(user_id, reference_data.target_id, &db_pool).await?;

    // 创建引用关系
    let reference_id = ResourceService::create_resource_reference(
        resource_id,
        reference_data.target_id,
        reference_data.reference_type,
        user_id,
        &db_pool,
    )
    .await?;

    Ok(success_response(serde_json::json!({
        "id": reference_id,
        "source_id": resource_id,
        "target_id": reference_data.target_id,
    })))
}

/// 删除资源引用关系
pub async fn delete_resource_reference(
    State(db_pool): State<SqlitePool>,
    Path((resource_id, target_id)): Path<(i64, i64)>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    // 验证源资源的所有权
    let _source = ResourceService::get_resource_by_id(user_id, resource_id, &db_pool).await?;

    // 删除引用关系
    let deleted =
        ResourceService::delete_resource_reference(resource_id, target_id, None, user_id, &db_pool)
            .await?;

    if !deleted {
        return Err(AppError::NotFound(
            "Resource reference not found".to_string(),
        ));
    }

    Ok(success_message_response(
        "Resource reference deleted successfully",
    ))
}

/// 获取资源的引用列表
/// 支持按引用类型过滤和分页
pub async fn get_resource_references(
    State(db_pool): State<SqlitePool>,
    Path(resource_id): Path<i64>,
    Query(query): Query<ResourceReferenceQuery>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    // 验证资源的所有权
    let _resource = ResourceService::get_resource_by_id(user_id, resource_id, &db_pool).await?;

    // 获取引用列表
    let references =
        ResourceService::get_resource_references(resource_id, query, user_id, &db_pool).await?;

    Ok(success_response(references))
}
