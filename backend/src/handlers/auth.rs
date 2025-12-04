use axum::{
    extract::{Json, State},
    response::Response,
};
use serde_json::{json, Value};
use sqlx::SqlitePool;
use tracing::info;

use crate::services::AuthService;
use crate::utils::error::AppError;
use crate::utils::response::{
    success_message_response, success_response, success_response_with_message,
};
use crate::{
    middleware::AuthenticatedUser,
    models::{ChangePassword, CreateUser, LoginUser, UserResponse},
};

pub async fn register(
    State(db_pool): State<SqlitePool>,
    Json(user_data): Json<CreateUser>,
) -> Result<Response, AppError> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::Internal("JWT secret not configured".to_string()))?;

    let auth_service = AuthService::new(jwt_secret);

    let user = auth_service.register(user_data, &db_pool).await?;
    let access_token = auth_service.generate_access_token(user.id)?;
    let refresh_token = auth_service.generate_refresh_token(user.id)?;

    Ok(success_response(json!({
        "user": UserResponse::from(user),
        "access_token": access_token,
        "refresh_token": refresh_token
    })))
}

pub async fn login(
    State(db_pool): State<SqlitePool>,
    Json(login_data): Json<LoginUser>,
) -> Result<Response, AppError> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::Internal("JWT secret not configured".to_string()))?;

    let auth_service = AuthService::new(jwt_secret);

    let user = auth_service.login(login_data, &db_pool).await?;
    let access_token = auth_service.generate_access_token(user.id)?;
    let refresh_token = auth_service.generate_refresh_token(user.id)?;

    Ok(success_response(json!({
        "user": UserResponse::from(user),
        "access_token": access_token,
        "refresh_token": refresh_token
    })))
}

pub async fn refresh_token(
    State(db_pool): State<SqlitePool>,
    Json(body): Json<Value>,
) -> Result<Response, AppError> {
    let refresh_token = body
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing refresh_token".to_string()))?;

    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::Internal("JWT secret not configured".to_string()))?;

    let auth_service = AuthService::new(jwt_secret);
    let user_id = auth_service.verify_token(refresh_token)?;

    // Verify user still exists and is active
    let user = auth_service
        .get_user_by_id(user_id, &db_pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

    let new_access_token = auth_service.generate_access_token(user.id)?;

    Ok(success_response(json!({
        "access_token": new_access_token
    })))
}

pub async fn get_current_user(
    State(db_pool): State<SqlitePool>,
    AuthenticatedUser(user_id): AuthenticatedUser,
) -> Result<Response, AppError> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::Internal("JWT secret not configured".to_string()))?;

    let auth_service = AuthService::new(jwt_secret);
    let user = auth_service
        .get_user_by_id(user_id, &db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if !user.is_active {
        return Err(AppError::Unauthorized("User is not active".to_string()));
    }

    Ok(success_response(json!({
        "user": UserResponse::from(user)
    })))
}

pub async fn change_password(
    State(db_pool): State<SqlitePool>,
    AuthenticatedUser(user_id): AuthenticatedUser,
    Json(password_data): Json<ChangePassword>,
) -> Result<Response, AppError> {
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::Internal("JWT secret not configured".to_string()))?;

    let auth_service = AuthService::new(jwt_secret);
    auth_service
        .change_password(
            user_id,
            password_data.current_password,
            password_data.new_password,
            &db_pool,
        )
        .await?;

    Ok(success_message_response("Password changed successfully"))
}

pub async fn logout(AuthenticatedUser(_user_id): AuthenticatedUser) -> Result<Response, AppError> {
    Ok(success_response_with_message(
        Value::Null,
        "Logout successful",
    ))
}
