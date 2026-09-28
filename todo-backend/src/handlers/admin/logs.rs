//! Admin log görüntüleme: filtreli, sayfalı, CSV/JSON export.
use actix_web::{get, web, HttpResponse};

use crate::dto::admin::{ExportQuery, LogQuery};
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::middleware::rbac::{require_min_role, require_permission};
use crate::models::profile::Role;
use crate::repositories::log_repo;
use crate::state::AppState;

/// GET /api/admin/logs?action=&severity=&user_id=&from=&to=&search=
#[utoipa::path(
    get,
    path = "/api/admin/logs",
    tag = "admin",
    params(LogQuery),
    responses(
        (status = 200, body = crate::dto::admin::LogListResponse),
        (status = 401),
        (status = 403),
    ),
    security(("bearer" = [])),
)]
#[get("/logs")]
pub async fn list_logs(
    state: web::Data<AppState>,
    user: AuthUser,
    q: web::Query<LogQuery>,
) -> Result<HttpResponse, AppError> {
    require_min_role(&user, Role::Moderator)?;
    require_permission(&state.db, &user, "log", "read").await?;
    let pg = crate::dto::pagination::Pagination::new(q.page, q.limit);
    let (rows, total) = log_repo::list(
        &state.db,
        q.action.as_deref(),
        q.severity.as_deref(),
        q.user_id.as_deref(),
        q.entity_type.as_deref(),
        q.from.as_deref(),
        q.to.as_deref(),
        q.search.as_deref(),
        pg.limit,
        pg.offset(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "data": rows, "page": pg.page, "limit": pg.limit, "total": total,
    })))
}

/// GET /api/admin/logs/export?format=csv|json — export izni gerekir.
#[utoipa::path(
    get,
    path = "/api/admin/logs/export",
    tag = "admin",
    params(ExportQuery),
    responses(
        (status = 200),
        (status = 401),
        (status = 403),
    ),
    security(("bearer" = [])),
)]
#[get("/logs/export")]
pub async fn export_logs(
    state: web::Data<AppState>,
    user: AuthUser,
    q: web::Query<ExportQuery>,
) -> Result<HttpResponse, AppError> {
    require_min_role(&user, Role::Moderator)?;
    require_permission(&state.db, &user, "log", "manage").await
        .or_else(|_| require_min_role(&user, Role::Admin))?;
    let format = q.format.as_deref().unwrap_or("json");
    let (rows, _) = log_repo::list(
        &state.db,
        q.action.as_deref(),
        q.severity.as_deref(),
        None,
        None,
        q.from.as_deref(),
        q.to.as_deref(),
        None,
        5000,
        0,
    )
    .await?;
    if format == "csv" {
        let mut w = csv::Writer::from_writer(vec![]);
        w.write_record(["id", "user_id", "actor_role", "action", "entity_type", "entity_id", "severity", "status", "created_at"])
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        for r in &rows {
            w.write_record([
                r.id.to_string(),
                r.user_id.map(|u| u.to_string()).unwrap_or_default(),
                r.actor_role.clone(),
                r.action.clone(),
                r.entity_type.clone(),
                r.entity_id.clone(),
                r.severity.clone(),
                r.status.clone(),
                r.created_at.to_rfc3339(),
            ])
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        }
        let bytes = w.into_inner().map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        return Ok(HttpResponse::Ok()
            .content_type("text/csv")
            .insert_header(("Content-Disposition", "attachment; filename=\"audit-logs.csv\""))
            .body(bytes));
    }
    Ok(HttpResponse::Ok().json(rows))
}
