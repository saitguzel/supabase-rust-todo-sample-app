//! Bildirim handler'ları (user scope).
use actix_web::{delete, get, patch, post, web, HttpResponse};
use uuid::Uuid;

use crate::dto::notification::NotificationQuery;
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::repositories::notification_repo;
use crate::state::AppState;

/// GET /api/notifications?unread=true
#[utoipa::path(
    get,
    path = "/api/notifications",
    tag = "notifications",
    params(NotificationQuery),
    responses(
        (status = 200, body = crate::dto::admin::NotificationListResponse),
        (status = 401),
    ),
    security(("bearer" = [])),
    operation_id = "list_notifications",
)]
#[get("")]
pub async fn list(
    state: web::Data<AppState>,
    user: AuthUser,
    q: web::Query<NotificationQuery>,
) -> Result<HttpResponse, AppError> {
    let pg = crate::dto::pagination::Pagination::new(q.page, q.limit);
    let (rows, total) = notification_repo::list(
        &state.db,
        user.id,
        q.unread.unwrap_or(false),
        pg.limit,
        pg.offset(),
    )
    .await?;
    let unread = notification_repo::unread_count(&state.db, user.id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "data": rows, "page": pg.page, "limit": pg.limit, "total": total, "unread": unread,
    })))
}

/// PATCH /api/notifications/:id/read
#[utoipa::path(
    patch,
    path = "/api/notifications/{id}/read",
    tag = "notifications",
    responses(
        (status = 204),
        (status = 401),
        (status = 404),
    ),
    security(("bearer" = [])),
)]
#[patch("/{id}/read")]
pub async fn mark_read(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let r = sqlx::query(
        "UPDATE notifications SET is_read = true, read_at = now() WHERE id = $1 AND user_id = $2",
    )
    .bind(*id)
    .bind(user.id)
    .execute(&state.db)
    .await?;
    if r.rows_affected() == 0 {
        return Err(AppError::NotFound("notification not found".into()));
    }
    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/notifications/read-all
#[utoipa::path(
    post,
    path = "/api/notifications/read-all",
    tag = "notifications",
    responses(
        (status = 204),
        (status = 401),
    ),
    security(("bearer" = [])),
)]
#[post("/read-all")]
pub async fn read_all(
    state: web::Data<AppState>,
    user: AuthUser,
) -> Result<HttpResponse, AppError> {
    sqlx::query(
        "UPDATE notifications SET is_read = true, read_at = now() WHERE user_id = $1 AND is_read = false",
    )
    .bind(user.id)
    .execute(&state.db)
    .await?;
    Ok(HttpResponse::NoContent().finish())
}

/// DELETE /api/notifications/:id
#[utoipa::path(
    delete,
    path = "/api/notifications/{id}",
    tag = "notifications",
    responses(
        (status = 204),
        (status = 401),
        (status = 404),
    ),
    security(("bearer" = [])),
    operation_id = "delete_notification",
)]
#[delete("/{id}")]
pub async fn delete(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    let r = sqlx::query("DELETE FROM notifications WHERE id = $1 AND user_id = $2")
        .bind(*id)
        .bind(user.id)
        .execute(&state.db)
        .await?;
    if r.rows_affected() == 0 {
        return Err(AppError::NotFound("notification not found".into()));
    }
    Ok(HttpResponse::NoContent().finish())
}

/// Bildirim route kapsamı — statik "/read-all" dinamik "/{id}"tan önce.
pub fn scope() -> actix_web::Scope {
    web::scope("/notifications")
        .service(read_all)
        .service(list)
        .service(mark_read)
        .service(delete)
}
