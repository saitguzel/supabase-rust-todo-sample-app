//! Auth handler'ları: register/login/refresh/logout/me.
use actix_web::{get, post, web, HttpRequest, HttpResponse};
use validator::Validate;

use crate::dto::auth::{LoginRequest, RefreshRequest, RegisterRequest};
use crate::errors::{validation_error, AppError};
use crate::middleware::auth::AuthUser;
use crate::repositories::user_repo;
use crate::services::auth_service;
use crate::state::AppState;

fn client_info(req: &HttpRequest) -> (Option<String>, Option<String>) {
    let ip = req
        .connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());
    let ua = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    (ip, ua)
}

/// POST /api/auth/register
#[utoipa::path(post, path = "/api/auth/register", request_body = RegisterRequest, responses((status = 201), (status = 400), (status = 409)))]
#[post("/register")]
pub async fn register(
    state: web::Data<AppState>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(validation_error)?;
    let (profile, tokens) = auth_service::register(&state.db, &state.config, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(serde_json::json!({
        "user": {"id": profile.id, "email": profile.email, "full_name": profile.full_name, "role": profile.role.as_str()},
        "tokens": tokens,
    })))
}

/// POST /api/auth/login
#[utoipa::path(post, path = "/api/auth/login", request_body = LoginRequest, responses((status = 200), (status = 401)))]
#[post("/login")]
pub async fn login(
    state: web::Data<AppState>,
    req: HttpRequest,
    body: web::Json<LoginRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(validation_error)?;
    let (ip, ua) = client_info(&req);
    let (profile, tokens) =
        auth_service::login(&state.db, &state.config, body.into_inner(), ip, ua).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user": {"id": profile.id, "email": profile.email, "full_name": profile.full_name, "role": profile.role.as_str()},
        "tokens": tokens,
    })))
}

/// POST /api/auth/refresh
#[utoipa::path(post, path = "/api/auth/refresh", request_body = RefreshRequest, responses((status = 200), (status = 401)))]
#[post("/refresh")]
pub async fn refresh(
    state: web::Data<AppState>,
    body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, AppError> {
    let tokens = auth_service::refresh(&state.db, &state.config, &body.refresh_token).await?;
    Ok(HttpResponse::Ok().json(tokens))
}

/// POST /api/auth/logout
#[utoipa::path(post, path = "/api/auth/logout", request_body = RefreshRequest, responses((status = 204)))]
#[post("/logout")]
pub async fn logout(
    state: web::Data<AppState>,
    body: web::Json<RefreshRequest>,
) -> Result<HttpResponse, AppError> {
    auth_service::logout(&state.db, &body.refresh_token).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// GET /api/auth/me
#[utoipa::path(get, path = "/api/auth/me", responses((status = 200), (status = 401)), security(("bearer" = [])))]
#[get("/me")]
pub async fn me(state: web::Data<AppState>, user: AuthUser) -> Result<HttpResponse, AppError> {
    let profile = user_repo::get_profile(&state.db, user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("profile not found".into()))?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "id": profile.id, "email": profile.email, "full_name": profile.full_name,
        "role": profile.role.as_str(), "avatar_url": profile.avatar_url,
    })))
}

/// Auth route kapsamı.
pub fn scope() -> actix_web::Scope {
    web::scope("/auth")
        .service(register)
        .service(login)
        .service(refresh)
        .service(logout)
        .service(me)
}
