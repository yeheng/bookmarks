use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::SqlitePool;

use crate::models::{CreateUser, LoginUser, User};
use crate::utils::error::{AppError, AppResult};
use crate::utils::jwt::JWTService;
use crate::utils::validation::{validate_email, validate_password, validate_username};

pub struct AuthService {
    jwt_service: JWTService,
}

impl AuthService {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_service: JWTService::new(jwt_secret),
        }
    }

    pub async fn register(&self, user_data: CreateUser, db_pool: &SqlitePool) -> AppResult<User> {
        // Validate input
        validate_username(&user_data.username).map_err(AppError::BadRequest)?;

        validate_email(&user_data.email)
            .then_some(())
            .ok_or_else(|| AppError::BadRequest("Invalid email format".to_string()))?;

        validate_password(&user_data.password).map_err(AppError::BadRequest)?;

        // Check if username or email already exists
        let existing = sqlx::query("SELECT id FROM users WHERE username = $1 OR email = $2")
            .bind(&user_data.username)
            .bind(&user_data.email)
            .fetch_optional(db_pool)
            .await?;

        if existing.is_some() {
            return Err(AppError::Conflict(
                "Username or email already exists".to_string(),
            ));
        }

        // Hash password
        let password_hash = hash(&user_data.password, DEFAULT_COST)?;

        // Create user
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, username, email, password_hash, avatar_url,
                      is_active, email_verified, email_verification_token,
                      password_reset_token, password_reset_expires_at,
                      last_login_at, created_at, updated_at
            "#,
        )
        .bind(&user_data.username)
        .bind(&user_data.email)
        .bind(password_hash)
        .fetch_one(db_pool)
        .await?;

        Ok(user)
    }

    pub async fn login(&self, login_data: LoginUser, db_pool: &SqlitePool) -> AppResult<User> {
        // Validate email format
        validate_email(&login_data.email)
            .then_some(())
            .ok_or_else(|| AppError::BadRequest("Invalid email format".to_string()))?;

        // Find user by email
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, avatar_url,
                   is_active, email_verified, email_verification_token,
                   password_reset_token, password_reset_expires_at,
                   last_login_at, created_at, updated_at
            FROM users
            WHERE email = $1 AND is_active = TRUE
            "#,
        )
        .bind(&login_data.email)
        .fetch_optional(db_pool)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

        // Verify password
        let is_valid = verify(&login_data.password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::Unauthorized(
                "Invalid email or password".to_string(),
            ));
        }

        // Update last login
        sqlx::query("UPDATE users SET last_login_at = CAST(strftime('%s', 'now') AS INTEGER) WHERE id = $1")
            .bind(user.id)
            .execute(db_pool)
            .await?;

        Ok(user)
    }

    pub fn generate_access_token(&self, user_id: i64) -> AppResult<String> {
        self.jwt_service.generate_access_token(user_id)
    }

    pub fn generate_refresh_token(&self, user_id: i64) -> AppResult<String> {
        self.jwt_service.generate_refresh_token(user_id)
    }

    pub fn verify_token(&self, token: &str) -> AppResult<i64> {
        self.jwt_service.verify_token(token)
    }

    pub async fn get_user_by_id(
        &self,
        user_id: i64,
        db_pool: &SqlitePool,
    ) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, avatar_url,
                   is_active, email_verified, email_verification_token,
                   password_reset_token, password_reset_expires_at,
                   last_login_at, created_at, updated_at
            FROM users
            WHERE id = $1 AND is_active = TRUE
            "#,
        )
        .bind(user_id)
        .fetch_optional(db_pool)
        .await?;

        Ok(user)
    }

    pub async fn change_password(
        &self,
        user_id: i64,
        current_password: String,
        new_password: String,
        db_pool: &SqlitePool,
    ) -> AppResult<()> {
        // Validate new password
        validate_password(&new_password).map_err(AppError::BadRequest)?;

        // Get current user
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, avatar_url,
                   is_active, email_verified, email_verification_token,
                   password_reset_token, password_reset_expires_at,
                   last_login_at, created_at, updated_at
            FROM users
            WHERE id = $1 AND is_active = TRUE
            "#,
        )
        .bind(user_id)
        .fetch_optional(db_pool)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Verify current password
        let is_valid = verify(&current_password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::Unauthorized(
                "Current password is incorrect".to_string(),
            ));
        }

        // Hash new password
        let new_password_hash = hash(&new_password, DEFAULT_COST)?;

        // Update password
        sqlx::query(
            "UPDATE users SET password_hash = $1, updated_at = CAST(strftime('%s', 'now') AS INTEGER) WHERE id = $2",
        )
        .bind(new_password_hash)
        .bind(user_id)
        .execute(db_pool)
        .await?;

        Ok(())
    }
}
