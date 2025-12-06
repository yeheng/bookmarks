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
    pub(crate) secret: String,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_service_new() {
        let service = JWTService::new("test_secret".to_string());
        assert_eq!(service.secret, "test_secret");
    }

    #[test]
    fn test_generate_access_token() {
        let service = JWTService::new("test_secret".to_string());
        let user_id = 123;

        let token = service.generate_access_token(user_id);
        assert!(token.is_ok());

        let token_str = token.unwrap();
        assert!(!token_str.is_empty());
    }

    #[test]
    fn test_generate_refresh_token() {
        let service = JWTService::new("test_secret".to_string());
        let user_id = 456;

        let token = service.generate_refresh_token(user_id);
        assert!(token.is_ok());

        let token_str = token.unwrap();
        assert!(!token_str.is_empty());
    }

    #[test]
    fn test_verify_valid_token() {
        let service = JWTService::new("test_secret".to_string());
        let user_id = 789;

        let token = service.generate_access_token(user_id).unwrap();
        let verified_id = service.verify_token(&token).unwrap();

        assert_eq!(verified_id, user_id);
    }

    #[test]
    fn test_verify_invalid_token() {
        let service = JWTService::new("test_secret".to_string());
        let invalid_token = "invalid.token.here";

        let result = service.verify_token(invalid_token);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_token_with_wrong_secret() {
        let service1 = JWTService::new("secret1".to_string());
        let service2 = JWTService::new("secret2".to_string());
        let user_id = 999;

        let token = service1.generate_access_token(user_id).unwrap();
        let result = service2.verify_token(&token);

        assert!(result.is_err());
    }

    #[test]
    fn test_token_contains_correct_claims() {
        let service = JWTService::new("test_secret".to_string());
        let user_id = 12345;

        let token = service.generate_access_token(user_id).unwrap();

        let token_data = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret(service.secret.as_ref()),
            &Validation::default(),
        )
        .unwrap();

        assert_eq!(token_data.claims.sub, user_id.to_string());
        assert!(token_data.claims.exp > token_data.claims.iat);
    }

    #[test]
    fn test_access_token_expiration() {
        let service = JWTService::new("test_secret".to_string());
        let user_id = 123;

        let token = service.generate_access_token(user_id).unwrap();
        let token_data = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret(service.secret.as_ref()),
            &Validation::default(),
        )
        .unwrap();

        let now = Utc::now().timestamp() as usize;
        let expected_exp = now + 15 * 60; // 15 minutes

        // Allow for some time difference (within 1 minute)
        assert!(token_data.claims.exp >= expected_exp - 60);
        assert!(token_data.claims.exp <= expected_exp + 60);
    }

    #[test]
    fn test_refresh_token_expiration() {
        let service = JWTService::new("test_secret".to_string());
        let user_id = 123;

        let token = service.generate_refresh_token(user_id).unwrap();
        let token_data = decode::<JwtClaims>(
            &token,
            &DecodingKey::from_secret(service.secret.as_ref()),
            &Validation::default(),
        )
        .unwrap();

        let now = Utc::now().timestamp() as usize;
        let expected_exp = now + 7 * 24 * 60 * 60; // 7 days

        // Allow for some time difference (within 1 minute)
        assert!(token_data.claims.exp >= expected_exp - 60);
        assert!(token_data.claims.exp <= expected_exp + 60);
    }
}
