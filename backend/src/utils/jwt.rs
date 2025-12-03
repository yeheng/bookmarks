use axum_jwt_auth::{Error as JwtAuthError, JwtDecoder};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

use crate::utils::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String, // User ID
    pub exp: usize,  // Expiration time
    pub iat: usize,  // Issued at time
}

#[derive(Clone)]
pub struct JWTService {
    secret: String,
}

impl JWTService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_access_token(&self, user_id: i64) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + Duration::minutes(15); // 15 minutes

        let claims = JwtClaims {
            sub: user_id.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )?;

        Ok(token)
    }

    pub fn generate_refresh_token(&self, user_id: i64) -> AppResult<String> {
        let now = Utc::now();
        let exp = now + Duration::days(7); // 7 days

        let claims = JwtClaims {
            sub: user_id.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> AppResult<i64> {
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &Validation::default(),
        )?;

        let user_id = token_data
            .claims
            .sub
            .parse::<i64>()
            .map_err(|_| AppError::Unauthorized("Invalid token format".to_string()))?;

        Ok(user_id)
    }
}

impl JwtDecoder<JwtClaims> for JWTService {
    fn decode<'a>(
        &'a self,
        token: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<TokenData<JwtClaims>, JwtAuthError>> + Send + 'a>> {
        let secret = self.secret.clone();

        Box::pin(async move {
            let data = decode::<JwtClaims>(
                token,
                &DecodingKey::from_secret(secret.as_ref()),
                &Validation::default(),
            )?;
            Ok(data)
        })
    }
}
