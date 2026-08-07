use chrono::{Duration, Utc};
use jsonwebtoken::{encode, decode, Algorithm, EncodingKey, DecodingKey, Header, Validation};
use jsonwebtoken::errors::Error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::ENV;

/// JWT Claims used for access tokens
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject: user id
    pub sub: String,
    /// Issued at (seconds since epoch)
    pub iat: i64,
    /// Expiration (seconds since epoch)
    pub exp: i64,
    /// Token type: e.g., "access"
    pub typ: String,
}

pub struct JwtService;

impl JwtService {
    /// Create a signed JWT access token for a user id with a specified ttl in minutes
    pub fn create_access_token(user_id: Uuid, ttl_minutes: i64) -> Result<String, Error> {
        let now = Utc::now();
        let exp = now + Duration::minutes(ttl_minutes);
        let claims = Claims {
            sub: user_id.to_string(),
            iat: now.timestamp(),
            exp: exp.timestamp(),
            typ: "access".to_string(),
        };

        // Use HS256 with APP_SECRET from config
        let mut header = Header::new(Algorithm::HS256);
        header.typ = Some("JWT".to_string());
        let key = EncodingKey::from_secret(ENV.app_secret.as_bytes());
        encode(&header, &claims, &key)
    }

    /// Verifies and decodes a JWT access token
    ///
    /// # Arguments
    /// * `token` - The JWT token string to verify
    ///
    /// # Returns
    /// * `Ok(Claims)` - The verified token claims containing user information
    /// * `Err(Error)` - If token is invalid, expired, or has wrong signature
    ///
    /// # Examples
    /// ```
    /// let claims = JwtService::verify_access_token(&token)?;
    /// let user_id = Uuid::parse_str(&claims.sub)?;
    /// ```
    pub fn verify_access_token(token: &str) -> Result<Claims, Error> {
        let key = DecodingKey::from_secret(ENV.app_secret.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_required_spec_claims(&["exp", "iat", "sub", "typ"]);
        validation.validate_exp = true;

        let token_data = decode::<Claims>(token, &key, &validation)?;
        Ok(token_data.claims)
    }
}
