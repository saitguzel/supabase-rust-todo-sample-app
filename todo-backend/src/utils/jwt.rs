//! JWT üretimi/doğrulaması (Supabase Auth uyumlu).
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::profile::Role;

/// JWT claim'leri — Supabase Auth ile uyumlu (sub, role, aud, iss).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aud: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
    pub token_type: String,
}

impl Claims {
    /// String rolü [`Role`] enum'una çevirir.
    pub fn role_enum(&self) -> Role {
        Role::parse(&self.role).unwrap_or(Role::User)
    }
}

/// Access token üretir.
pub fn create_access_token(
    user_id: Uuid,
    email: &str,
    role: &Role,
    secret: &str,
    ttl_minutes: i64,
    aud: Option<&str>,
    iss: Option<&str>,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.as_str().to_string(),
        iat: now,
        exp: now + ttl_minutes * 60,
        aud: aud.map(|s| s.to_string()),
        iss: iss.map(|s| s.to_string()),
        token_type: "access".to_string(),
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))
}

/// Refresh token üretir (opaque değil, JWT; hash'i sessions tablosunda tutulur).
pub fn create_refresh_token(
    user_id: Uuid,
    email: &str,
    role: &Role,
    secret: &str,
    ttl_days: i64,
) -> Result<String, AppError> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.as_str().to_string(),
        iat: now,
        exp: now + ttl_days * 86400,
        aud: None,
        iss: None,
        token_type: "refresh".to_string(),
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))
}

/// Token doğrular; Supabase JWT'si de aynı secret ile doğrulanabilir.
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, AppError> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    // aud/iss opsiyonel — Supabase projesine göre config'den gelir.
    validation.validate_aud = false;
    decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &validation)
        .map(|d| d.claims)
        .map_err(|_| AppError::Unauthorized("invalid or expired token".into()))
}
