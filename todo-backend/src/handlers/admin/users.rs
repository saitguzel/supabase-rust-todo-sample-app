//! Admin kullanıcı yönetimi: listele, rol ata, ban/unban.
use actix_web::{get, patch, web, HttpResponse};
use uuid::Uuid;
use validator::Validate;

use crate::dto::admin::{UpdateRoleRequest, UpdateStatusRequest, UserQuery};
use crate::errors::{validation_error, AppError};
use crate::middleware::auth::AuthUser;
use crate::middleware::rbac::{require_min_role, require_permission};
use crate::models::profile::Role;
use crate::repositories::notification_repo;
use crate::services::audit_service;
use crate::state::AppState;

/// GET /api/admin/users?role=&active=&search=
#[utoipa::path(
    get,
    path = "/api/admin/users",
    tag = "admin",
    params(UserQuery),
    responses(
        (status = 200, body = crate::dto::admin::UserListResponse),
        (status = 401),
        (status = 403),
    ),
    security(("bearer" = [])),
)]
#[get("/users")]
pub async fn list_users(
    state: web::Data<AppState>,
    user: AuthUser,
    q: web::Query<UserQuery>,
) -> Result<HttpResponse, AppError> {
    require_min_role(&user, Role::Moderator)?;
    require_permission(&state.db, &user, "user", "read").await?;
    let pg = crate::dto::pagination::Pagination::new(q.page, q.limit);
    let rows: Vec<crate::models::profile::Profile> = sqlx::query_as(
        r#"SELECT id, email, full_name, avatar_url, role,
                  is_active, last_login_at, created_at, updated_at
           FROM profiles
           WHERE ($1 IS NULL OR role::text = $1)
             AND ($2 IS NULL OR is_active = $2)
             AND ($3 IS NULL OR email ILIKE '%'||$3||'%' OR full_name ILIKE '%'||$3||'%')
           ORDER BY created_at DESC LIMIT $4 OFFSET $5"#,
    )
    .bind(q.role.clone())
    .bind(q.active)
    .bind(q.search.clone())
    .bind(pg.limit)
    .bind(pg.offset())
    .fetch_all(&state.db)
    .await?;
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM profiles WHERE ($1 IS NULL OR role::text = $1) AND ($2 IS NULL OR is_active = $2)",
    )
    .bind(q.role.clone())
    .bind(q.active)
    .fetch_one(&state.db)
    .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "data": rows, "page": pg.page, "limit": pg.limit, "total": total,
    })))
}

/// PATCH /api/admin/users/:id/role — yalnızca admin+.
#[utoipa::path(
    patch,
    path = "/api/admin/users/{id}/role",
    tag = "admin",
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, body = crate::dto::admin::MutationResponse),
        (status = 400),
        (status = 401),
        (status = 403),
        (status = 404),
    ),
    security(("bearer" = [])),
)]
#[patch("/users/{id}/role")]
pub async fn update_role(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<Uuid>,
    body: web::Json<UpdateRoleRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(validation_error)?;
    require_min_role(&user, Role::Admin)?;
    require_permission(&state.db, &user, "user", "manage").await?;
    let new_role = Role::parse(&body.role)
        .ok_or_else(|| AppError::Validation("invalid role".into()))?;
    // super_admin ataması yalnızca super_admin tarafından.
    if new_role == Role::SuperAdmin && user.role != Role::SuperAdmin {
        audit_service::log_denied(
            state.db.clone(),
            Some(user.id),
            user.role.as_str().into(),
            "user".into(),
            id.to_string(),
        );
        return Err(AppError::Forbidden("only super_admin can assign super_admin".into()));
    }
    // Kendini düşürmeyi engelle (lockout koruması).
    if *id == user.id {
        return Err(AppError::Validation("cannot change own role".into()));
    }
    let old: Option<String> =
        sqlx::query_scalar("SELECT role::text FROM profiles WHERE id = $1")
            .bind(*id)
            .fetch_optional(&state.db)
            .await?;
    let Some(old_role) = old else {
        return Err(AppError::NotFound("user not found".into()));
    };
    sqlx::query("UPDATE profiles SET role = $2::user_role WHERE id = $1")
        .bind(*id)
        .bind(new_role.as_str())
        .execute(&state.db)
        .await?;
    // Rol değişiminde bildirim + detaylı audit.
    let _ = notification_repo::create(
        &state.db,
        *id,
        crate::models::notification::NotificationType::RoleChanged,
        "Rolün güncellendi",
        &format!("Rolün '{old_role}' → '{}' olarak değiştirildi.", new_role.as_str()),
        &serde_json::json!({"old": old_role, "new": new_role.as_str()}),
    )
    .await;
    audit_service::log_event(
        &state.db,
        Some(user.id),
        user.role.as_str(),
        "ROLE_CHANGE",
        "user",
        &id.to_string(),
        Some(serde_json::json!({"role": old_role})),
        Some(serde_json::json!({"role": new_role.as_str()})),
        None,
        "info",
        "success",
    )
    .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"id": *id, "role": new_role.as_str()})))
}

/// PATCH /api/admin/users/:id/status (ban/unban) — admin+.
#[utoipa::path(
    patch,
    path = "/api/admin/users/{id}/status",
    tag = "admin",
    request_body = UpdateStatusRequest,
    responses(
        (status = 200, body = crate::dto::admin::MutationResponse),
        (status = 400),
        (status = 401),
        (status = 403),
        (status = 404),
    ),
    security(("bearer" = [])),
)]
#[patch("/users/{id}/status")]
pub async fn update_status(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<Uuid>,
    body: web::Json<UpdateStatusRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(validation_error)?;
    require_min_role(&user, Role::Admin)?;
    require_permission(&state.db, &user, "user", "update").await?;
    if *id == user.id {
        return Err(AppError::Validation("cannot change own status".into()));
    }
    let r = sqlx::query("UPDATE profiles SET is_active = $2 WHERE id = $1")
        .bind(*id)
        .bind(body.is_active)
        .execute(&state.db)
        .await?;
    if r.rows_affected() == 0 {
        return Err(AppError::NotFound("user not found".into()));
    }
    Ok(HttpResponse::Ok().json(serde_json::json!({"id": *id, "is_active": body.is_active})))
}
