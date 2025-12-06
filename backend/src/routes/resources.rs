use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::handlers::resources::{
    batch_update_resources, create_resource, delete_resource, get_resource,
    get_resources, update_resource, create_resource_reference,
    delete_resource_reference, get_resource_references,
};
use crate::state::AppState;

/// 配置资源相关的路由
/// 包括基本的 CRUD 操作和资源引用管理
pub fn resource_routes() -> Router<AppState> {
    Router::new()
        // 基础资源操作
        .route("/", get(get_resources))
        .route("/", post(create_resource))
        .route("/batch", post(batch_update_resources))
        .route("/:id", get(get_resource))
        .route("/:id", put(update_resource))
        .route("/:id", delete(delete_resource))
        // 资源引用管理
        .route("/:id/references", post(create_resource_reference))
        .route("/:id/references", get(get_resource_references))
        .route("/:id/references/:target_id", delete(delete_resource_reference))
}
