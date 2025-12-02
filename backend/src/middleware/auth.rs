use axum::{
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use axum_jwt_auth::{AuthError, Claims as JwtClaimsExtractor};
use uuid::Uuid;

use crate::state::AppState;
use crate::utils::error::AppError;
use crate::utils::jwt::JwtClaims;

pub async fn auth_middleware(
    State(app_state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = request.into_parts();

    let claims = JwtClaimsExtractor::<JwtClaims>::from_request_parts(&mut parts, &app_state)
        .await
        .map_err(map_auth_error)?;

    let user_id = Uuid::parse_str(&claims.claims.sub)
        .map_err(|_| AppError::Unauthorized("Invalid token subject".to_string()))?;

    // Verify user exists and is active
    let user_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM users WHERE id = $1 AND is_active = TRUE)")
            .bind(user_id)
            .fetch_one(&app_state.db_pool)
            .await?;

    if !user_exists {
        return Err(AppError::Unauthorized(
            "User not found or inactive".to_string(),
        ));
    }

    // Add user ID to request extensions
    parts.extensions.insert(user_id);
    let request = Request::from_parts(parts, body);

    Ok(next.run(request).await)
}

// 自定义 Extractor：自动从 request extensions 提取 user_id
#[derive(Debug, Clone, Copy)]
pub struct AuthenticatedUser(pub Uuid);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Uuid>()
            .copied()
            .map(AuthenticatedUser)
            .ok_or_else(|| AppError::Unauthorized("User not authenticated".to_string()))
    }
}

fn map_auth_error(err: AuthError) -> AppError {
    match err {
        AuthError::MissingToken => AppError::Unauthorized("Missing authentication token".into()),
        AuthError::ExpiredSignature => AppError::Unauthorized("Token expired".into()),
        AuthError::InvalidToken
        | AuthError::InvalidSignature
        | AuthError::InvalidAudience
        | AuthError::InvalidIssuer
        | AuthError::InvalidSubject
        | AuthError::ImmatureSignature
        | AuthError::InvalidAlgorithm
        | AuthError::MissingAlgorithm
        | AuthError::MissingRequiredClaim(_)
        | AuthError::InternalError => AppError::Unauthorized("Invalid authentication token".into()),
    }
}
