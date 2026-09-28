//! Todo handler'ları (user scope CRUD + share + bulk + complete).
use actix_web::{delete, get, patch, post, web, HttpResponse};
use uuid::Uuid;
use validator::Validate;

use crate::dto::todo::{
    BulkTodoRequest, CreateTodoRequest, Paginated, ReorderRequest, ShareTodoRequest, TodoQuery,
    UpdateTodoRequest,
};
use crate::errors::{validation_error, AppError};
use crate::middleware::auth::AuthUser;
use crate::models::todo::{TodoPriority, TodoStatus};
use crate::services::todo_service;
use crate::state::AppState;
use crate::utils::pagination::decode_cursor;

fn parse_status(s: Option<String>) -> Result<Option<TodoStatus>, AppError> {
    match s.as_deref() {
        None => Ok(None),
        Some("pending") => Ok(Some(TodoStatus::Pending)),
        Some("in_progress") => Ok(Some(TodoStatus::InProgress)),
        Some("completed") => Ok(Some(TodoStatus::Completed)),
        Some("archived") => Ok(Some(TodoStatus::Archived)),
        Some(other) => Err(AppError::Validation(format!("invalid status: {other}"))),
    }
}

fn parse_priority(s: Option<String>) -> Result<Option<TodoPriority>, AppError> {
    match s.as_deref() {
        None => Ok(None),
        Some("low") => Ok(Some(TodoPriority::Low)),
        Some("medium") => Ok(Some(TodoPriority::Medium)),
        Some("high") => Ok(Some(TodoPriority::High)),
        Some("urgent") => Ok(Some(TodoPriority::Urgent)),
        Some(other) => Err(AppError::Validation(format!("invalid priority: {other}"))),
    }
}

/// GET /api/todos
#[utoipa::path(
    get,
    path = "/api/todos",
    tag = "todos",
    params(TodoQuery),
    responses(
        (status = 200, body = PaginatedTodo),
        (status = 401),
    ),
    security(("bearer" = [])),
    operation_id = "list_todos",
)]
#[get("")]
pub async fn list(
    state: web::Data<AppState>,
    user: AuthUser,
    q: web::Query<TodoQuery>,
) -> Result<HttpResponse, AppError> {
    let pg = crate::dto::pagination::Pagination::new(q.page, q.limit);
    let cursor = q.cursor.as_deref().and_then(decode_cursor);
    let (rows, total) = todo_service::list(
        &state.db,
        &user,
        parse_status(q.status.clone())?,
        parse_priority(q.priority.clone())?,
        q.search.clone(),
        pg.limit,
        pg.offset(),
        cursor,
    )
    .await?;
    let has_more = rows.len() as i64 > pg.limit;
    let mut rows = rows;
    rows.truncate(pg.limit as usize);
    let next_cursor = if has_more {
        rows.last().map(|t| crate::utils::pagination::encode_cursor(&t.created_at, &t.id))
    } else {
        None
    };
    Ok(HttpResponse::Ok().json(Paginated {
        data: rows,
        page: pg.page,
        limit: pg.limit,
        total,
        next_cursor,
    }))
}

/// POST /api/todos
#[utoipa::path(
    post,
    path = "/api/todos",
    tag = "todos",
    request_body = CreateTodoRequest,
    responses(
        (status = 201, body = Todo),
        (status = 400),
        (status = 401),
    ),
    security(("bearer" = [])),
)]
#[post("")]
pub async fn create(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<CreateTodoRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(validation_error)?;
    let todo = todo_service::create(&state.db, &user, body.into_inner()).await?;
    Ok(HttpResponse::Created().json(todo))
}

/// GET /api/todos/:id
#[utoipa::path(
    get,
    path = "/api/todos/{id}",
    tag = "todos",
    responses(
        (status = 200, body = Todo),
        (status = 401),
        (status = 404),
    ),
    security(("bearer" = [])),
)]
#[get("/{id}")]
pub async fn get_one(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(todo_service::get(&state.db, &user, *id).await?))
}

/// PATCH /api/todos/:id
#[utoipa::path(
    patch,
    path = "/api/todos/{id}",
    tag = "todos",
    request_body = UpdateTodoRequest,
    responses(
        (status = 200, body = Todo),
        (status = 400),
        (status = 401),
        (status = 404),
    ),
    security(("bearer" = [])),
)]
#[patch("/{id}")]
pub async fn update(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<Uuid>,
    body: web::Json<UpdateTodoRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(validation_error)?;
    Ok(HttpResponse::Ok()
        .json(todo_service::update(&state.db, &user, *id, body.into_inner()).await?))
}

/// DELETE /api/todos/:id (soft delete)
#[utoipa::path(
    delete,
    path = "/api/todos/{id}",
    tag = "todos",
    responses(
        (status = 204),
        (status = 401),
        (status = 403),
        (status = 404),
    ),
    security(("bearer" = [])),
    operation_id = "delete_todo",
)]
#[delete("/{id}")]
pub async fn delete(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    todo_service::delete(&state.db, &user, *id).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/todos/:id/complete
#[utoipa::path(
    post,
    path = "/api/todos/{id}/complete",
    tag = "todos",
    responses(
        (status = 200, body = Todo),
        (status = 401),
        (status = 403),
        (status = 404),
    ),
    security(("bearer" = [])),
)]
#[post("/{id}/complete")]
pub async fn complete(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, AppError> {
    Ok(HttpResponse::Ok().json(todo_service::complete(&state.db, &user, *id).await?))
}

/// POST /api/todos/:id/share
#[utoipa::path(
    post,
    path = "/api/todos/{id}/share",
    tag = "todos",
    request_body = ShareTodoRequest,
    responses(
        (status = 204),
        (status = 400),
        (status = 401),
        (status = 403),
        (status = 404),
    ),
    security(("bearer" = [])),
)]
#[post("/{id}/share")]
pub async fn share(
    state: web::Data<AppState>,
    user: AuthUser,
    id: web::Path<Uuid>,
    body: web::Json<ShareTodoRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(validation_error)?;
    todo_service::share(
        &state.db,
        &user,
        *id,
        &body.email,
        body.permission.as_deref().unwrap_or("read"),
    )
    .await?;
    Ok(HttpResponse::NoContent().finish())
}

/// POST /api/todos/bulk — routes.rs/scope içinde önce kaydedilir ki "/{id}" ile çakışmasın.
#[utoipa::path(
    post,
    path = "/api/todos/bulk",
    tag = "todos",
    request_body = BulkTodoRequest,
    responses(
        (status = 200, body = crate::dto::admin::UpdatedCountResponse),
        (status = 400),
        (status = 401),
    ),
    security(("bearer" = [])),
)]
#[post("/bulk")]
pub async fn bulk_alt(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<BulkTodoRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(validation_error)?;
    let n = todo_service::bulk(&state.db, &user, &body.ids, &body.action).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"updated": n})))
}

/// POST /api/todos/reorder — sürükle-bırak sıralaması ({ids: [hedef sıra]}).
#[utoipa::path(
    post,
    path = "/api/todos/reorder",
    tag = "todos",
    request_body = ReorderRequest,
    responses(
        (status = 200, body = crate::dto::admin::UpdatedCountResponse),
        (status = 400),
        (status = 401),
    ),
    security(("bearer" = [])),
)]
#[post("/reorder")]
pub async fn reorder(
    state: web::Data<AppState>,
    user: AuthUser,
    body: web::Json<ReorderRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate().map_err(validation_error)?;
    let n = todo_service::reorder(&state.db, &user, &body.ids).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"updated": n})))
}

/// Todo route kapsamı.
pub fn scope() -> actix_web::Scope {
    web::scope("/todos")
        .service(bulk_alt)
        .service(reorder)
        .service(list)
        .service(create)
        .service(complete)
        .service(share)
        .service(get_one)
        .service(update)
        .service(delete)
}
