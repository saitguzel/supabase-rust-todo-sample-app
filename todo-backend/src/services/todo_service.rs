//! Todo servis mantığı: sahiplik + paylaşım + RBAC.
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::dto::todo::{CreateTodoRequest, UpdateTodoRequest};
use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::profile::Role;
use crate::models::todo::{Todo, TodoPriority, TodoStatus};
use crate::repositories::{notification_repo, todo_repo};

/// Liste — admin/mod tümünü görebilir, user yalnızca kendininkini.
#[allow(clippy::too_many_arguments)]
pub async fn list(
    db: &PgPool,
    user: &AuthUser,
    status: Option<TodoStatus>,
    priority: Option<TodoPriority>,
    search: Option<String>,
    limit: i64,
    offset: i64,
    cursor: Option<crate::utils::pagination::Cursor>,
) -> Result<(Vec<Todo>, i64), AppError> {
    let admin_scope = matches!(user.role, Role::Admin | Role::SuperAdmin | Role::Moderator);
    todo_repo::list(
        db,
        user.id,
        admin_scope,
        status,
        priority,
        search.as_deref(),
        limit,
        offset,
        cursor.as_ref().map(|c| c.created_at),
        cursor.as_ref().map(|c| c.id),
    )
    .await
}

/// Oluşturur + audit için todo döner.
pub async fn create(
    db: &PgPool,
    user: &AuthUser,
    req: CreateTodoRequest,
) -> Result<Todo, AppError> {
    todo_repo::create(
        db,
        user.id,
        &req.title,
        req.description.as_deref().unwrap_or(""),
        req.status.unwrap_or(TodoStatus::Pending),
        req.priority.unwrap_or(TodoPriority::Medium),
        req.due_date,
        req.tags.as_deref().unwrap_or(&[]),
    )
    .await
}

/// Okur — erişim kontrolü ile.
pub async fn get(db: &PgPool, user: &AuthUser, id: Uuid) -> Result<Todo, AppError> {
    let todo = todo_repo::get(db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("todo not found".into()))?;
    if todo.user_id == user.id {
        return Ok(todo);
    }
    if matches!(user.role, Role::Admin | Role::SuperAdmin | Role::Moderator) {
        return Ok(todo);
    }
    if todo_repo::has_access(db, id, user.id).await? {
        return Ok(todo);
    }
    Err(AppError::Forbidden("no access to todo".into()))
}

/// Günceller — sahip veya write-paylaşımlı veya admin.
pub async fn update(
    db: &PgPool,
    user: &AuthUser,
    id: Uuid,
    req: UpdateTodoRequest,
) -> Result<Todo, AppError> {
    let existing = get(db, user, id).await?;
    let is_owner = existing.user_id == user.id;
    let is_admin = matches!(user.role, Role::Admin | Role::SuperAdmin);
    if !is_owner && !is_admin {
        // Paylaşım izni yazma mı?
        let write: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM todo_shares WHERE todo_id = $1 AND shared_with_user_id = $2 AND permission = 'write'",
        )
        .bind(id)
        .bind(user.id)
        .fetch_one(db)
        .await?;
        if write == 0 {
            return Err(AppError::Forbidden("write access required".into()));
        }
    }
    let due = match req.due_date {
        None => None,
        Some(v) => Some(Some(v)),
    };
    todo_repo::update(
        db,
        id,
        req.title.as_deref(),
        req.description.as_deref(),
        req.status,
        req.priority,
        due,
        req.tags.as_deref(),
    )
    .await
}

/// Siler (soft) — sahip veya admin.
pub async fn delete(db: &PgPool, user: &AuthUser, id: Uuid) -> Result<(), AppError> {
    let existing = get(db, user, id).await?;
    if existing.user_id != user.id && !matches!(user.role, Role::Admin | Role::SuperAdmin) {
        return Err(AppError::Forbidden("only owner or admin can delete".into()));
    }
    todo_repo::soft_delete(db, id).await?;
    Ok(())
}

/// Tamamlandı olarak işaretler + sahibine bildirim yazar.
pub async fn complete(db: &PgPool, user: &AuthUser, id: Uuid) -> Result<Todo, AppError> {
    let todo = update(
        db,
        user,
        id,
        UpdateTodoRequest {
            title: None,
            description: None,
            status: Some(TodoStatus::Completed),
            priority: None,
            due_date: None,
            tags: None,
        },
    )
    .await?;
    let _ = notification_repo::create(
        db,
        todo.user_id,
        crate::models::notification::NotificationType::TodoCompleted,
        "Todo tamamlandı",
        &format!("'{}' tamamlandı.", todo.title),
        &serde_json::json!({"todo_id": todo.id}),
    )
    .await;
    Ok(todo)
}

/// Paylaşır + karşı tarafa bildirim yazar.
pub async fn share(
    db: &PgPool,
    user: &AuthUser,
    id: Uuid,
    email: &str,
    permission: &str,
) -> Result<(), AppError> {
    let todo = get(db, user, id).await?;
    if todo.user_id != user.id && !matches!(user.role, Role::Admin | Role::SuperAdmin) {
        return Err(AppError::Forbidden("only owner can share".into()));
    }
    let perm = match permission {
        "write" => "write",
        _ => "read",
    };
    let target: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM profiles WHERE email = $1")
            .bind(email.to_lowercase())
            .fetch_optional(db)
            .await?;
    let Some((target_id,)) = target else {
        return Err(AppError::NotFound("shared user not found".into()));
    };
    todo_repo::share(db, id, target_id, perm).await?;
    let _ = notification_repo::create(
        db,
        target_id,
        crate::models::notification::NotificationType::TodoAssigned,
        "Todo paylaşıldı",
        &format!("'{}' başlıklı todo seninle paylaşıldı.", todo.title),
        &serde_json::json!({"todo_id": id, "permission": perm}),
    )
    .await;
    Ok(())
}

/// Bulk aksiyon: complete / archive / delete.
pub async fn bulk(db: &PgPool, user: &AuthUser, ids: &[Uuid], action: &str) -> Result<u64, AppError> {
    let mut n = 0u64;
    for id in ids {
        let r = match action {
            "complete" => complete(db, user, *id).await.map(|_| 1).map_err(|e| e),
            "archive" => update(
                db,
                user,
                *id,
                UpdateTodoRequest {
                    title: None,
                    description: None,
                    status: Some(TodoStatus::Archived),
                    priority: None,
                    due_date: None,
                    tags: None,
                },
            )
            .await
            .map(|_| 1),
            "delete" => delete(db, user, *id).await.map(|_| 1),
            _ => return Err(AppError::Validation("unknown bulk action".into())),
        };
        if r.is_ok() {
            n += 1;
        }
    }
    Ok(n)
}

/// Sıralamayı kaydeder — yalnızca sahibi sıralayabilir.
pub async fn reorder(db: &PgPool, user: &AuthUser, ids: &[Uuid]) -> Result<u64, AppError> {
    todo_repo::reorder(db, user.id, ids).await
}

/// Due-date geçen todo'ları bulur (cron worker kullanır).
pub async fn find_due(
    db: &PgPool,
    hours: i64,
) -> Result<Vec<(Uuid, Uuid, String, Option<DateTime<Utc>>)>, AppError> {
    Ok(sqlx::query_as(
        r#"SELECT id, user_id, title, due_date FROM todos
           WHERE is_deleted = false AND status <> 'completed'
             AND due_date IS NOT NULL AND due_date <= now() + ($1 || ' hours')::interval"#,
    )
    .bind(hours.to_string())
    .fetch_all(db)
    .await?)
}
