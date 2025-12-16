use axum::{
    routing::post,
    Router,
};

use crate::handlers::command::handle_command;
use crate::state::AppState;

/// 配置命令模式路由
/// 提供统一的命令处理入口点
pub fn command_routes() -> Router<AppState> {
    Router::new()
        // 统一命令处理端点
        .route("/execute", post(handle_command))
}