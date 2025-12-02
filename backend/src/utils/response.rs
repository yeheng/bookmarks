use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub fn success_response<T: serde::Serialize>(data: T) -> Response {
    Json(json!({
        "success": true,
        "data": data
    }))
    .into_response()
}

pub fn success_message_response(message: &str) -> Response {
    Json(json!({
        "success": true,
        "message": message
    }))
    .into_response()
}

pub fn success_response_with_message<T: serde::Serialize>(data: T, message: &str) -> Response {
    Json(json!({
        "success": true,
        "data": data,
        "message": message
    }))
    .into_response()
}

pub fn error_response(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(json!({
            "success": false,
            "error": message
        })),
    )
        .into_response()
}

pub fn validation_error_response(errors: Vec<String>) -> Response {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "success": false,
            "error": "Validation failed",
            "details": errors
        })),
    )
        .into_response()
}
