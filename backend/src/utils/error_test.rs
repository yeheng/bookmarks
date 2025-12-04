use axum::{http::StatusCode, response::IntoResponse};

use crate::utils::error::AppError;

#[test]
fn test_app_error_database() {
    let sqlx_error = sqlx::Error::PoolClosed;
    let app_error = AppError::Database(sqlx_error);
    
    let response = app_error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_app_error_unauthorized() {
    let app_error = AppError::Unauthorized("Invalid credentials".to_string());
    
    let response = app_error.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_app_error_conflict() {
    let app_error = AppError::Conflict("User already exists".to_string());
    
    let response = app_error.into_response();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[test]
fn test_app_error_bad_request() {
    let app_error = AppError::BadRequest("Invalid input data".to_string());
    
    let response = app_error.into_response();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test]
fn test_app_error_not_found() {
    let app_error = AppError::NotFound("User not found".to_string());
    
    let response = app_error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_app_error_internal() {
    let app_error = AppError::Internal("Something went wrong".to_string());
    
    let response = app_error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_app_error_jwt() {
    use jsonwebtoken::errors::ErrorKind;
    let jwt_error = jsonwebtoken::errors::Error::from(ErrorKind::InvalidToken);
    let app_error = AppError::Jwt(jwt_error);
    
    let response = app_error.into_response();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn test_app_error_bcrypt() {
    use bcrypt::BcryptError;
    let bcrypt_error = BcryptError::InvalidCost("Invalid cost".to_string());
    let app_error = AppError::Bcrypt(bcrypt_error);
    
    let response = app_error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_app_error_config() {
    use config::ConfigError;
    let config_error = ConfigError::Message("Config error".to_string());
    let app_error = AppError::Config(config_error);
    
    let response = app_error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_app_error_serialization() {
    let json_error = serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::Other, "IO error"));
    let app_error = AppError::Serialization(json_error);
    
    let response = app_error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_app_error_anyhow() {
    let anyhow_error = anyhow::anyhow!("Some error");
    let app_error = AppError::Anyhow(anyhow_error);
    
    let response = app_error.into_response();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_app_error_from_anyhow_with_app_error() {
    let original_app_error = AppError::NotFound("Not found".to_string());
    let anyhow_error = anyhow::Error::new(original_app_error);
    let app_error = AppError::from(anyhow_error);
    
    let response = app_error.into_response();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test]
fn test_app_error_display() {
    let errors = vec![
        (AppError::Database(sqlx::Error::PoolClosed), "Database error: pool closed"),
        (AppError::Unauthorized("test".to_string()), "Authentication error: test"),
        (AppError::Conflict("test".to_string()), "Conflict error: test"),
        (AppError::BadRequest("test".to_string()), "Bad request error: test"),
        (AppError::NotFound("test".to_string()), "Not found error: test"),
        (AppError::Internal("test".to_string()), "Internal server error: test"),
    ];
    
    for (error, expected) in errors {
        assert_eq!(error.to_string(), expected);
    }
}

#[test]
fn test_app_error_from_sqlx_error() {
    let sqlx_error = sqlx::Error::PoolClosed;
    let app_error: AppError = sqlx_error.into();
    
    match app_error {
        AppError::Database(_) => assert!(true),
        _ => assert!(false, "Expected Database error"),
    }
}

#[test]
fn test_app_error_from_jwt_error() {
    use jsonwebtoken::errors::ErrorKind;
    let jwt_error = jsonwebtoken::errors::Error::from(ErrorKind::InvalidToken);
    let app_error: AppError = jwt_error.into();
    
    match app_error {
        AppError::Jwt(_) => assert!(true),
        _ => assert!(false, "Expected Jwt error"),
    }
}

#[test]
fn test_app_error_from_bcrypt_error() {
    use bcrypt::BcryptError;
    let bcrypt_error = BcryptError::InvalidCost("test".to_string());
    let app_error: AppError = bcrypt_error.into();
    
    match app_error {
        AppError::Bcrypt(_) => assert!(true),
        _ => assert!(false, "Expected Bcrypt error"),
    }
}

#[test]
fn test_app_error_from_config_error() {
    use config::ConfigError;
    let config_error = ConfigError::Message("test".to_string());
    let app_error: AppError = config_error.into();
    
    match app_error {
        AppError::Config(_) => assert!(true),
        _ => assert!(false, "Expected Config error"),
    }
}

#[test]
fn test_app_error_from_serialization_error() {
    let json_error = serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
    let app_error: AppError = json_error.into();
    
    match app_error {
        AppError::Serialization(_) => assert!(true),
        _ => assert!(false, "Expected Serialization error"),
    }
}