use axum::{
    extract::{Json, State},
    response::{IntoResponse, Response},
    Json as AxumJson,
};
use serde_json::json;
use tracing::{error, info};

use crate::middleware::AuthenticatedUser;
use crate::models::{
    Action, CommandRequest, CommandResponse, CreateCollection, CreateResource, CreateTag,
    ResourceBatchRequest, ResourceQuery, StatsPeriod, UpdateResource,
};
use crate::services::{
    CollectionService, ResourceService, StatsService, TagService,
};
use crate::state::AppState;
use crate::utils::error::AppError;

/// 命令处理器 - 处理所有来自前端的命令请求
pub async fn handle_command(
    State(app_state): State<AppState>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(command_request): Json<CommandRequest>,
) -> Result<Response, AppError> {
    info!("收到命令请求: action={:?}, user_id={}", command_request.action, user_id);

    let request_id = command_request.request_id.clone().unwrap_or_default();

    // 执行命令并处理结果
    match execute_command(command_request, user_id, &app_state).await {
        Ok(data) => {
            info!("命令执行成功: action={:?}, request_id={}", data.action, request_id);
            let command_response = CommandResponse::success_with_request_id(Some(data.response), request_id);
            Ok(AxumJson(command_response).into_response())
        }
        Err(e) => {
            error!("命令执行失败: action={:?}, error={}, request_id={}", e.action, e.error_message, request_id);
            let command_response = CommandResponse::error_with_request_id(
                e.error_code,
                e.error_message,
                e.error_details,
                request_id,
            );
            Ok(AxumJson(command_response).into_response())
        }
    }
}

/// 命令执行结果
struct CommandResult {
    action: Action,
    response: serde_json::Value,
}

/// 命令执行错误
struct CommandExecutionError {
    action: Action,
    error_code: String,
    error_message: String,
    error_details: Option<serde_json::Value>,
}

impl From<AppError> for CommandExecutionError {
    fn from(error: AppError) -> Self {
        let (code, message) = match &error {
            AppError::BadRequest(msg) => ("BAD_REQUEST".to_string(), msg.clone()),
            AppError::Unauthorized(msg) => ("UNAUTHORIZED".to_string(), msg.clone()),
            AppError::NotFound(msg) => ("NOT_FOUND".to_string(), msg.clone()),
            AppError::Conflict(msg) => ("CONFLICT".to_string(), msg.clone()),
            AppError::Internal(msg) => ("INTERNAL_ERROR".to_string(), msg.clone()),
            AppError::Database(_) => ("DATABASE_ERROR".to_string(), "Database operation failed".to_string()),
            AppError::Jwt(_) => ("JWT_ERROR".to_string(), "Invalid authentication token".to_string()),
            AppError::Bcrypt(_) => ("PASSWORD_ERROR".to_string(), "Password processing failed".to_string()),
            AppError::Config(_) => ("CONFIG_ERROR".to_string(), "Configuration error".to_string()),
            AppError::Serialization(_) => ("SERIALIZATION_ERROR".to_string(), "Data serialization failed".to_string()),
            AppError::Anyhow(_) => ("UNKNOWN_ERROR".to_string(), "An unknown error occurred".to_string()),
        };

        Self {
            action: Action::GetResources, // 默认值，会在使用时被覆盖
            error_code: code,
            error_message: message,
            error_details: None,
        }
    }
}

/// 执行具体的命令
async fn execute_command(
    command: CommandRequest,
    user_id: i64,
    app_state: &AppState,
) -> Result<CommandResult, CommandExecutionError> {
    match command.action {
        // 资源管理命令
        Action::GetResources => {
            let params: ResourceQuery = if command.params.is_null() {
                ResourceQuery::default()
            } else {
                command.get_params().map_err(|e| CommandExecutionError {
                    action: Action::GetResources,
                    error_code: "INVALID_PARAMS".to_string(),
                    error_message: format!("参数解析失败: {}", e),
                    error_details: None,
                })?
            };

            let resources = ResourceService::get_resources(user_id, params, &app_state.db_pool)
                .await
                .map_err(|e| {
                    let mut cmd_error: CommandExecutionError = e.into();
                    cmd_error.action = Action::GetResources;
                    cmd_error
                })?;

            Ok(CommandResult {
                action: Action::GetResources,
                response: json!(resources),
            })
        }

        Action::GetResource => {
            let resource_id: i64 = command.get_param("id").map_err(|e| CommandExecutionError {
                action: Action::GetResource,
                error_code: "INVALID_PARAMS".to_string(),
                error_message: format!("id参数解析失败: {}", e),
                error_details: None,
            })?;

            let resource = ResourceService::get_resource_by_id(user_id, resource_id, &app_state.db_pool)
                .await
                .map_err(|e| {
                    let mut cmd_error: CommandExecutionError = e.into();
                    cmd_error.action = Action::GetResource;
                    cmd_error
                })?;

            Ok(CommandResult {
                action: Action::GetResource,
                response: json!(resource),
            })
        }

        Action::CreateResource => {
            let create_data: CreateResource = command.get_params().map_err(|e| CommandExecutionError {
                action: Action::CreateResource,
                error_code: "INVALID_PARAMS".to_string(),
                error_message: format!("创建资源参数解析失败: {}", e),
                error_details: None,
            })?;

            let resource = ResourceService::create_resource(user_id, create_data, &app_state.db_pool)
                .await
                .map_err(|e| {
                    let mut cmd_error: CommandExecutionError = e.into();
                    cmd_error.action = Action::CreateResource;
                    cmd_error
                })?;

            Ok(CommandResult {
                action: Action::CreateResource,
                response: json!(resource),
            })
        }

        Action::UpdateResource => {
            let resource_id: i64 = command.get_param("id").map_err(|e| CommandExecutionError {
                action: Action::UpdateResource,
                error_code: "INVALID_PARAMS".to_string(),
                error_message: format!("id参数解析失败: {}", e),
                error_details: None,
            })?;

            let update_data: UpdateResource = command.get_param("data").map_err(|e| CommandExecutionError {
                action: Action::UpdateResource,
                error_code: "INVALID_PARAMS".to_string(),
                error_message: format!("更新数据参数解析失败: {}", e),
                error_details: None,
            })?;

            let resource = ResourceService::update_resource(
                user_id,
                resource_id,
                update_data,
                &app_state.db_pool,
            )
            .await
            .map_err(|e| {
                let mut cmd_error: CommandExecutionError = e.into();
                cmd_error.action = Action::UpdateResource;
                cmd_error
            })?;

            Ok(CommandResult {
                action: Action::UpdateResource,
                response: json!(resource),
            })
        }

        Action::DeleteResource => {
            let resource_id: i64 = command.get_param("id").map_err(|e| CommandExecutionError {
                action: Action::DeleteResource,
                error_code: "INVALID_PARAMS".to_string(),
                error_message: format!("id参数解析失败: {}", e),
                error_details: None,
            })?;

            let deleted = ResourceService::delete_resource(user_id, resource_id, &app_state.db_pool)
                .await
                .map_err(|e| {
                    let mut cmd_error: CommandExecutionError = e.into();
                    cmd_error.action = Action::DeleteResource;
                    cmd_error
                })?;

            if !deleted {
                return Err(CommandExecutionError {
                    action: Action::DeleteResource,
                    error_code: "NOT_FOUND".to_string(),
                    error_message: "资源未找到".to_string(),
                    error_details: None,
                });
            }

            Ok(CommandResult {
                action: Action::DeleteResource,
                response: json!({"message": "资源删除成功"}),
            })
        }

        Action::BatchUpdateResources => {
            let batch_request: ResourceBatchRequest = command.get_params().map_err(|e| {
                CommandExecutionError {
                    action: Action::BatchUpdateResources,
                    error_code: "INVALID_PARAMS".to_string(),
                    error_message: format!("批量更新参数解析失败: {}", e),
                    error_details: None,
                }
            })?;

            let result = ResourceService::batch_process(user_id, batch_request, &app_state.db_pool)
                .await
                .map_err(|e| {
                    let mut cmd_error: CommandExecutionError = e.into();
                    cmd_error.action = Action::BatchUpdateResources;
                    cmd_error
                })?;

            Ok(CommandResult {
                action: Action::BatchUpdateResources,
                response: json!(result),
            })
        }

        // 搜索命令
        Action::SearchResources => {
            let params: ResourceQuery = if command.params.is_null() {
                ResourceQuery::default()
            } else {
                command.get_params().map_err(|e| CommandExecutionError {
                    action: Action::SearchResources,
                    error_code: "INVALID_PARAMS".to_string(),
                    error_message: format!("搜索参数解析失败: {}", e),
                    error_details: None,
                })?
            };

            let resources = ResourceService::get_resources(user_id, params, &app_state.db_pool)
                .await
                .map_err(|e| {
                    let mut cmd_error: CommandExecutionError = e.into();
                    cmd_error.action = Action::SearchResources;
                    cmd_error
                })?;

            Ok(CommandResult {
                action: Action::SearchResources,
                response: json!(resources),
            })
        }

        // 统计命令
        Action::GetUserStats => {
            let stats = StatsService::get_user_stats(user_id, StatsPeriod::default(), &app_state.db_pool)
                .await
                .map_err(|e| {
                    let mut cmd_error: CommandExecutionError = e.into();
                    cmd_error.action = Action::GetUserStats;
                    cmd_error
                })?;

            Ok(CommandResult {
                action: Action::GetUserStats,
                response: json!(stats),
            })
        }

        // 收藏夹管理命令
        Action::GetCollections => {
            use crate::models::CollectionQuery;
            let collections = CollectionService::get_collections(
                user_id,
                CollectionQuery {
                    limit: None,
                    offset: None,
                    parent_id: None,
                    is_public: None,
                },
                &app_state.db_pool
            )
                .await
                .map_err(|e| {
                    let mut cmd_error: CommandExecutionError = e.into();
                    cmd_error.action = Action::GetCollections;
                    cmd_error
                })?;

            Ok(CommandResult {
                action: Action::GetCollections,
                response: json!(collections),
            })
        }

        Action::CreateCollection => {
            let create_data: CreateCollection = command.get_params().map_err(|e| {
                CommandExecutionError {
                    action: Action::CreateCollection,
                    error_code: "INVALID_PARAMS".to_string(),
                    error_message: format!("创建收藏夹参数解析失败: {}", e),
                    error_details: None,
                }
            })?;

            let collection = CollectionService::create_collection(user_id, create_data, &app_state.db_pool)
                .await
                .map_err(|e| {
                    let mut cmd_error: CommandExecutionError = e.into();
                    cmd_error.action = Action::CreateCollection;
                    cmd_error
                })?;

            Ok(CommandResult {
                action: Action::CreateCollection,
                response: json!(collection),
            })
        }

        // 标签管理命令
        Action::GetTags => {
            use crate::models::TagQuery;
            let tags = TagService::get_tags(
                user_id,
                TagQuery {
                    limit: None,
                    offset: None,
                    search: None,
                },
                &app_state.db_pool
            )
                .await
                .map_err(|e| {
                    let mut cmd_error: CommandExecutionError = e.into();
                    cmd_error.action = Action::GetTags;
                    cmd_error
                })?;

            Ok(CommandResult {
                action: Action::GetTags,
                response: json!(tags),
            })
        }

        Action::CreateTag => {
            let create_data: CreateTag = command.get_params().map_err(|e| CommandExecutionError {
                action: Action::CreateTag,
                error_code: "INVALID_PARAMS".to_string(),
                error_message: format!("创建标签参数解析失败: {}", e),
                error_details: None,
            })?;

            let tag = TagService::create_tag(user_id, create_data, &app_state.db_pool)
                .await
                .map_err(|e| {
                    let mut cmd_error: CommandExecutionError = e.into();
                    cmd_error.action = Action::CreateTag;
                    cmd_error
                })?;

            Ok(CommandResult {
                action: Action::CreateTag,
                response: json!(tag),
            })
        }

        // 未实现的命令
        action => {
            let action_clone = action.clone();
            Err(CommandExecutionError {
                action,
                error_code: "NOT_IMPLEMENTED".to_string(),
                error_message: format!("命令 {:?} 尚未实现", action_clone),
                error_details: None,
            })
        },
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Action, CommandRequest, CommandResponse};
    use serde_json::json;

    #[test]
    fn test_command_request_parsing() {
        let params = json!({
            "id": 42,
            "data": {
                "title": "Updated Title",
                "description": "Updated description"
            }
        });

        let command = CommandRequest::_new(Action::UpdateResource, params);

        assert_eq!(command.action, Action::UpdateResource);

        let id: i64 = command.get_param("id").unwrap();
        assert_eq!(id, 42);
    }

    #[test]
    fn test_command_response_format() {
        // 模拟后端返回的 CommandResponse 格式
        let backend_response = CommandResponse::success_with_request_id(
            Some(json!({
                "items": [{"id": 1, "title": "Test Resource"}],
                "pagination": {"total": 1, "page": 1}
            })),
            "test_request_id".to_string(),
        );

        // 验证前端期望的格式结构
        assert_eq!(backend_response.success, true);
        assert!(backend_response.data.is_some());
        assert_eq!(backend_response.request_id, Some("test_request_id".to_string()));
        assert!(backend_response.error.is_none());
        assert!(backend_response.timestamp > 0);

        // 验证数据结构符合前端 PaginatedApiResponse 期望
        let data = backend_response.data.unwrap();
        assert!(data.get("items").is_some());
        assert!(data.get("pagination").is_some());
    }

    #[test]
    fn test_command_error_format() {
        // 模拟错误响应格式
        let error_response = CommandResponse::error_with_request_id(
            "NOT_FOUND".to_string(),
            "Resource not found".to_string(),
            Some(json!({"resource_id": 123})),
            "error_request_id".to_string(),
        );

        // 验证错误格式
        assert_eq!(error_response.success, false);
        assert!(error_response.data.is_none());
        assert!(error_response.error.is_some());

        let error = error_response.error.unwrap();
        assert_eq!(error.code, "NOT_FOUND");
        assert_eq!(error.message, "Resource not found");
        assert!(error.details.is_some());
    }

    #[test]
    fn test_command_request_format() {
        // 模拟前端发送的命令请求格式
        let frontend_request = CommandRequest::_new(
            Action::GetResources,
            json!({
                "limit": 20,
                "offset": 0,
                "search": "test"
            })
        );

        assert_eq!(frontend_request.action, Action::GetResources);
        assert!(frontend_request.request_id.is_some());
        assert!(frontend_request.params.get("limit").is_some());
        assert!(frontend_request.params.get("search").is_some());
    }
}