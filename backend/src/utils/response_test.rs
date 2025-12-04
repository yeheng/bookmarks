use axum::{http::StatusCode, response::IntoResponse};

use crate::utils::response::*;

#[test]
fn test_success_response() {
    let data = serde_json::json!({
        "id": 1,
        "name": "test"
    });
    
    let response = success_response(data);
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
fn test_success_message_response() {
    let response = success_message_response("Operation successful");
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
fn test_success_response_with_message() {
    let data = serde_json::json!({
        "id": 123,
        "username": "testuser"
    });
    
    let response = success_response_with_message(data, "User created successfully");
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
fn test_error_response() {
    let response = error_response(StatusCode::NOT_FOUND, "Resource not found");
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_validation_error_response() {
    let errors = vec![
        "Username must be at least 3 characters".to_string(),
        "Email is invalid".to_string(),
    ];
    
    let response = validation_error_response(errors);
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_error_response_different_status_codes() {
    let test_cases = vec![
        (StatusCode::BAD_REQUEST, "Bad request"),
        (StatusCode::UNAUTHORIZED, "Unauthorized"),
        (StatusCode::FORBIDDEN, "Forbidden"),
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
    ];
    
    for (status_code, message) in test_cases {
        let response = error_response(status_code, message);
        assert_eq!(response.status(), status_code);
    }
}

#[test]
fn test_success_response_with_complex_data() {
    #[derive(serde::Serialize)]
    struct User {
        id: i64,
        username: String,
        email: String,
        is_active: bool,
    }
    
    let user = User {
        id: 42,
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        is_active: true,
    };
    
    let response = success_response(user);
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
fn test_success_message_response_empty_string() {
    let response = success_message_response("");
    assert_eq!(response.status(), StatusCode::OK);
}

#[test]
fn test_validation_error_response_empty() {
    let errors: Vec<String> = vec![];
    
    let response = validation_error_response(errors);
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}