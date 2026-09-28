//! Auth servis mantığı: register/login/refresh/logout.
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::config::AppConfig;
use crate::dto::auth::{LoginRequest, RegisterRequest, TokenResponse};
use crate::errors::AppError;
use crate::models::profile::Profile;
use crate::repositories::user_repo;
use crate::utils::jwt::{create_access_token, create_refresh_token, verify_token};
use crate::utils::password::{hash_password, hash_token, verify_password};

/// Kayıt olur, profil döner + token çifti üretir.
pub async fn register(
    db: &PgPool,
    cfg: &AppConfig,
    req: RegisterRequest,
) -> Result<(Profile, TokenResponse), AppError> {
    let hash = hash_password(&req.password)?;
    let profile = user_repo::create_user(db, &req.email, &hash, &req.full_name).await?;
    let tokens = issue_tokens(db, cfg, &profile, None, None).await?;
    Ok((profile, tokens))
}

/// Giriş yapar; pasif kullanıcı engellenir.
pub async fn login(
    db: &PgPool,
    cfg: &AppConfig,
    req: LoginRequest,
    ip: Option<String>,
    ua: Option<String>,
) -> Result<(Profile, TokenResponse), AppError> {
    let found = user_repo::find_auth_with_profile(db, &req.email).await?;
    let Some((uid, hash, profile)) = found else {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    };
    if !profile.is_active {
        return Err(AppError::Forbidden("account is disabled".into()));
    }
    if !verify_password(&req.password, &hash) {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }
    let _ = uid;
    user_repo::touch_login(db, profile.id).await?;
    let tokens = issue_tokens(db, cfg, &profile, ip, ua).await?;
    Ok((profile, tokens))
}

/// Refresh token ile yeni token çifti üretir (rotasyon).
pub async fn refresh(
    db: &PgPool,
    cfg: &AppConfig,
    refresh_token: &str,
) -> Result<TokenResponse, AppError> {
    let claims = verify_token(refresh_token, &cfg.jwt_secret)?;
    if claims.token_type != "refresh" {
        return Err(AppError::Unauthorized("not a refresh token".into()));
    }
    let hash = hash_token(refresh_token);
    let row: Option<(Uuid, chrono::DateTime<Utc>)> =
        sqlx::query_as("SELECT user_id, expires_at FROM sessions WHERE refresh_token_hash = $1")
            .bind(&hash)
            .fetch_optional(db)
            .await?;
    let Some((user_id, expires_at)) = row else {
        return Err(AppError::Unauthorized("session not found".into()));
    };
    if expires_at < Utc::now() {
        sqlx::query("DELETE FROM sessions WHERE refresh_token_hash = $1")
            .bind(&hash)
            .execute(db)
            .await?;
        return Err(AppError::Unauthorized("session expired".into()));
    }
    let profile = user_repo::get_profile(db, user_id)
        .await?
        .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;
    // Rotasyon: eskisini sil.
    sqlx::query("DELETE FROM sessions WHERE refresh_token_hash = $1")
        .bind(&hash)
        .execute(db)
        .await?;
    issue_tokens(db, cfg, &profile, None, None).await
}

/// Çıkış — refresh oturumunu siler.
pub async fn logout(db: &PgPool, refresh_token: &str) -> Result<(), AppError> {
    let hash = hash_token(refresh_token);
    sqlx::query("DELETE FROM sessions WHERE refresh_token_hash = $1")
        .bind(hash)
        .execute(db)
        .await?;
    Ok(())
}

/// Token çifti üretir + session satırı yazar.
async fn issue_tokens(
    db: &PgPool,
    cfg: &AppConfig,
    profile: &Profile,
    ip: Option<String>,
    ua: Option<String>,
) -> Result<TokenResponse, AppError> {
    let access = create_access_token(
        profile.id,
        &profile.email,
        &profile.role,
        &cfg.jwt_secret,
        cfg.jwt_access_ttl_minutes,
        cfg.jwt_audience.as_deref(),
        cfg.jwt_issuer.as_deref(),
    )?;
    let refresh = create_refresh_token(
        profile.id,
        &profile.email,
        &profile.role,
        &cfg.jwt_secret,
        cfg.jwt_refresh_ttl_days,
    )?;
    let expires_at = Utc::now() + chrono::Duration::days(cfg.jwt_refresh_ttl_days);
    sqlx::query(
        "INSERT INTO sessions (user_id, refresh_token_hash, ip, user_agent, expires_at) VALUES ($1,$2,$3::inet,$4,$5)",
    )
    .bind(profile.id)
    .bind(hash_token(&refresh))
    .bind(ip.as_deref())
    .bind(ua.as_deref())
    .bind(expires_at)
    .execute(db)
    .await?;
    Ok(TokenResponse {
        access_token: access,
        refresh_token: refresh,
        token_type: "Bearer".into(),
        expires_in: cfg.jwt_access_ttl_minutes * 60,
    })
}
