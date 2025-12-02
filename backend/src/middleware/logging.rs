use axum::{extract::Request, middleware::Next, response::Response};
use tracing::{info, warn};

pub async fn logging_middleware(request: Request, next: Next) -> Response {
    // 在移动 request 之前提取所有需要的信息
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string(); // 转换为owned String，避免借用request

    let start = std::time::Instant::now();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();

    if status.is_server_error() {
        warn!(
            method = %method,
            uri = %uri,
            status = %status,
            duration = ?duration,
            user_agent = user_agent,
            "Request completed with server error"
        );
    } else {
        info!(
            method = %method,
            uri = %uri,
            status = %status,
            duration = ?duration,
            user_agent = user_agent,
            "Request completed"
        );
    }

    response
}
