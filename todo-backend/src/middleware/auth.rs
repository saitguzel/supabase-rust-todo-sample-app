//! AuthMiddleware — JWT doğrular, `AuthUser` extractor'ı sağlar.
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use futures_util::future::{ready, Ready};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::profile::Role;
use crate::utils::jwt::{verify_token, Claims};

/// Doğrulanmış kullanıcı (request extractor).
#[derive(Debug, Clone)]
#[allow(dead_code)] // email/claims audit ve ilerideki handler'lar için taşınır.
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub role: Role,
    pub claims: Claims,
}

impl AuthUser {
    /// Rol hiyerarşisine göre en az bir role sahip mi?
    pub fn has_any_role(&self, roles: &[Role]) -> bool {
        roles.iter().any(|r| r.as_str() == self.role.as_str())
    }

    /// En az belirtilen seviyede mi? (örn. admin >= moderator)
    pub fn has_min_rank(&self, min: Role) -> bool {
        self.role.rank() >= min.rank()
    }
}

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let result = (|| -> Result<AuthUser, AppError> {
            let header = req
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .ok_or_else(|| AppError::Unauthorized("missing Authorization header".into()))?;
            let token = header
                .strip_prefix("Bearer ")
                .ok_or_else(|| AppError::Unauthorized("expected Bearer token".into()))?;
            // Secret AppState'ten alınır (app_data yoksa env fallback).
            let secret = req
                .app_data::<actix_web::web::Data<crate::state::AppState>>()
                .map(|s| s.config.jwt_secret.clone())
                .unwrap_or_else(|| {
                    std::env::var("SUPABASE_JWT_SECRET")
                        .or_else(|_| std::env::var("JWT_SECRET"))
                        .unwrap_or_default()
                });
            let claims = verify_token(token, &secret)?;
            if claims.token_type != "access" {
                return Err(AppError::Unauthorized("not an access token".into()));
            }
            Ok(AuthUser {
                id: claims.sub,
                email: claims.email.clone(),
                role: claims.role_enum(),
                claims,
            })
        })();
        ready(result.map_err(|e| e.into()))
    }
}

/// Opsiyonel auth — token yoksa None döner, varsa doğrular.
/// Genel (public) endpoint'lerde kişiselleştirme için kullanılır.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MaybeAuthUser(pub Option<AuthUser>);

impl FromRequest for MaybeAuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let has_header = req.headers().contains_key("authorization");
        if !has_header {
            return ready(Ok(MaybeAuthUser(None)));
        }
        // Header varsa zorunlu doğrulama için AuthUser'ı yeniden kullanamayız
        // (Payload tüketimi yok — manuel parse).
        let header = req
            .headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let token = header.strip_prefix("Bearer ").unwrap_or("");
        let secret = req
            .app_data::<actix_web::web::Data<crate::state::AppState>>()
            .map(|s| s.config.jwt_secret.clone())
            .unwrap_or_default();
        match verify_token(token, &secret) {
            Ok(claims) => ready(Ok(MaybeAuthUser(Some(AuthUser {
                id: claims.sub,
                email: claims.email.clone(),
                role: claims.role_enum(),
                claims,
            })))),
            Err(e) => ready(Err(e.into())),
        }
    }
}
