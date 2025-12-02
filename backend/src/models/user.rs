use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub email_verification_token: Option<String>,
    pub password_reset_token: Option<String>,
    pub password_reset_expires_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            avatar_url: user.avatar_url,
            is_active: user.is_active,
            email_verified: user.email_verified,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChangePassword {
    pub current_password: String,
    pub new_password: String,
}
