//! Admin istatistikleri + broadcast + ayarlar.
use actix_web::{get, patch, post, web, HttpResponse};
use validator::Validate;

use crate::dto::admin::UpdateSettingsRequest;
use crate::dto::notification::BroadcastRequest;
use crate::errors::{validation_error, AppError};
use crate::middleware::auth::AuthUser;
use crate::middleware::rbac::{require_min_role, require_permission, require_roles};
use crate::models::profile::Role;
use crate::services::notification_service;
use crate::state::AppState;

/// GET /api/admin/stats — dashboard metrikleri.
#[utoipa::path(
    get,
    path = "/api/admin/stats",
    tag = "admin",
    responses(
        (status = 200, body = StatsResponse),
        (status = 401),
        (status = 403),
    ),
    security(("bearer" = [])),
)]
#[get("/stats")]
pub async fn stats(state: web::Data<AppState>, user: AuthUser) -> Result<HttpResponse, AppError> {
    require_min_role(&user, Role::Moderator)?;
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM profiles")
        .fetch_one(&state.db)
        .await?;
    let active_users: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM profiles WHERE is_active = true")
            .fetch_one(&state.db)
            .await?;
    let total_todos: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM todos WHERE is_deleted = false")
            .fetch_one(&state.db)
            .await?;
    let completed_todos: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM todos WHERE is_deleted = false AND status = 'completed'",
    )
    .fetch_one(&state.db)
    .await?;
    let overdue_todos: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM todos WHERE is_deleted = false AND status <> 'completed' AND due_date IS NOT NULL AND due_date < now()",
    )
    .fetch_one(&state.db)
    .await?;
    let unread_notifications: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE is_read = false")
            .fetch_one(&state.db)
            .await?;
    let audit_last_24h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_logs WHERE created_at > now() - interval '24 hours'",
    )
    .fetch_one(&state.db)
    .await?;
    Ok(HttpResponse::Ok().json(crate::dto::admin::StatsResponse {
        total_users,
        active_users,
        total_todos,
        completed_todos,
        overdue_todos,
        unread_notifications,
        audit_last_24h,
    }))
}

/// POST /api/admin/notifications/broadcast — admin+.
#[utoipa::path(
    post,
    path = "/api/admin/notifications/broadcast",
    tag = "admin",
    request_body = BroadcastRequest,
    responses(
        (status = 200, body = crate::dto::admin::BroadcastResponse),
        (status = 400),
        (status = 401),
        (status = 403),
    ),
    security(("bearer" = [])),
)]
#[post("/notifications/broadcast")]
pub async fn broadcast(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<BroadcastRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(validation_error)?;
    // Kesin rol listesi (hiyerarşi değil): yalnızca admin kademesi yayın yapar.
    require_roles(&user, &[Role::Admin, Role::SuperAdmin])?;
    require_permission(&state.db, &user, "notification", "manage").await?;
    let n = notification_service::broadcast(&state.db, &body.title, &body.message).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"sent": n})))
}

/// GET /api/admin/settings
#[utoipa::path(
    get,
    path = "/api/admin/settings",
    tag = "admin",
    responses(
        (status = 200, body = Vec<crate::dto::admin::SettingEntry>),
        (status = 401),
        (status = 403),
    ),
    security(("bearer" = [])),
)]
#[get("/settings")]
pub async fn get_settings(
    state: web::Data<AppState>,
    user: AuthUser,
) -> Result<HttpResponse, AppError> {
    require_min_role(&user, Role::Admin)?;
    let rows: Vec<(String, serde_json::Value)> =
        sqlx::query_as("SELECT key, value FROM system_settings ORDER BY key")
            .fetch_all(&state.db)
            .await?;
    Ok(HttpResponse::Ok().json(rows.iter().map(|(k, v)| serde_json::json!({"key": k, "value": v})).collect::<Vec<_>>()))
}

/// PATCH /api/admin/settings/:key
#[utoipa::path(
    patch,
    path = "/api/admin/settings/{key}",
    tag = "admin",
    request_body = UpdateSettingsRequest,
    responses(
        (status = 200, body = crate::dto::admin::SettingEntry),
        (status = 400),
        (status = 401),
        (status = 403),
    ),
    security(("bearer" = [])),
)]
#[patch("/settings/{key}")]
pub async fn update_settings(
    state: web::Data<AppState>,
    user: AuthUser,
    key: web::Path<String>,
    body: web::Json<UpdateSettingsRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(validation_error)?;
    require_min_role(&user, Role::Admin)?;
    sqlx::query(
        "INSERT INTO system_settings (key, value, updated_by) VALUES ($1,$2,$3)
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_by = EXCLUDED.updated_by, updated_at = now()",
    )
    .bind(key.as_str())
    .bind(&body.value)
    .bind(user.id)
    .execute(&state.db)
    .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"key": key.as_str(), "value": body.value})))
}
