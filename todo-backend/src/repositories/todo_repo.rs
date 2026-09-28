//! Todo repository — parametrik sorgular (SQL injection korumalı).
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::todo::{Todo, TodoPriority, TodoStatus};

/// Filtreli todo listesi (cursor keyset + offset fallback).
#[allow(clippy::too_many_arguments)]
pub async fn list(
    db: &PgPool,
    user_id: Uuid,
    is_admin_scope: bool,
    status: Option<TodoStatus>,
    priority: Option<TodoPriority>,
    search: Option<&str>,
    limit: i64,
    offset: i64,
    cursor_created: Option<DateTime<Utc>>,
    cursor_id: Option<Uuid>,
) -> Result<(Vec<Todo>, i64), AppError> {
    let status_s = status.map(|s| match s {
        TodoStatus::Pending => "pending",
        TodoStatus::InProgress => "in_progress",
        TodoStatus::Completed => "completed",
        TodoStatus::Archived => "archived",
    });
    let priority_s = priority.map(|p| match p {
        TodoPriority::Low => "low",
        TodoPriority::Medium => "medium",
        TodoPriority::High => "high",
        TodoPriority::Urgent => "urgent",
    });

    let rows = sqlx::query_as::<_, Todo>(
        r#"SELECT id, user_id, title, description,
                  status,
                  priority,
                  due_date, completed_at, tags, is_deleted, position, created_at, updated_at
           FROM todos
           WHERE (NOT $1 OR user_id = $2)
             AND ($3::todo_status IS NULL OR status = $3::todo_status)
             AND ($4::todo_priority IS NULL OR priority = $4::todo_priority)
             AND ($5 IS NULL OR title ILIKE '%' || $5 || '%' OR description ILIKE '%' || $5 || '%')
             AND is_deleted = false
             AND ($6::timestamptz IS NULL OR (created_at, id) < ($6, $7))
           ORDER BY position ASC, created_at DESC, id DESC
           LIMIT $8 OFFSET $9"#,
    )
    .bind(is_admin_scope)
    .bind(user_id)
    .bind(status_s)
    .bind(priority_s)
    .bind(search)
    .bind(cursor_created)
    .bind(cursor_id)
    .bind(limit + 1)
    .bind(offset)
    .fetch_all(db)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM todos
           WHERE (NOT $1 OR user_id = $2) AND is_deleted = false
             AND ($3::todo_status IS NULL OR status = $3::todo_status)"#,
    )
    .bind(is_admin_scope)
    .bind(user_id)
    .bind(status_s)
    .fetch_one(db)
    .await?;

    // limit+1 ile next-page var mı bakılır; handler kırpar.
    Ok((rows, total))
}

/// Tek todo (sahiplik + paylaşım kontrolü handler'da yapılır).
pub async fn get(db: &PgPool, id: Uuid) -> Result<Option<Todo>, AppError> {
    Ok(sqlx::query_as::<_, Todo>(
        r#"SELECT id, user_id, title, description,
                  status,
                  priority,
                  due_date, completed_at, tags, is_deleted, position, created_at, updated_at
           FROM todos WHERE id = $1 AND is_deleted = false"#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?)
}

/// Yeni todo oluşturur.
pub async fn create(
    db: &PgPool,
    user_id: Uuid,
    title: &str,
    description: &str,
    status: TodoStatus,
    priority: TodoPriority,
    due_date: Option<DateTime<Utc>>,
    tags: &[String],
) -> Result<Todo, AppError> {
    let status_s = match status {
        TodoStatus::Pending => "pending",
        TodoStatus::InProgress => "in_progress",
        TodoStatus::Completed => "completed",
        TodoStatus::Archived => "archived",
    };
    let priority_s = match priority {
        TodoPriority::Low => "low",
        TodoPriority::Medium => "medium",
        TodoPriority::High => "high",
        TodoPriority::Urgent => "urgent",
    };
    Ok(sqlx::query_as::<_, Todo>(
        r#"INSERT INTO todos (user_id, title, description, status, priority, due_date, tags, position)
           VALUES ($1,$2,$3,$4::todo_status,$5::todo_priority,$6,$7,
                   COALESCE((SELECT MAX(position) + 1 FROM todos WHERE user_id = $1), 0))
           RETURNING id, user_id, title, description,
                     status,
                     priority,
                     due_date, completed_at, tags, is_deleted, position, created_at, updated_at"#,
    )
    .bind(user_id)
    .bind(title)
    .bind(description)
    .bind(status_s)
    .bind(priority_s)
    .bind(due_date)
    .bind(tags)
    .fetch_one(db)
    .await?)
}

/// Todo günceller (kısmi — None alanlar korunur).
#[allow(clippy::too_many_arguments)]
pub async fn update(
    db: &PgPool,
    id: Uuid,
    title: Option<&str>,
    description: Option<&str>,
    status: Option<TodoStatus>,
    priority: Option<TodoPriority>,
    due_date: Option<Option<DateTime<Utc>>>,
    tags: Option<&[String]>,
) -> Result<Todo, AppError> {
    let status_s = status.map(|s| match s {
        TodoStatus::Pending => "pending",
        TodoStatus::InProgress => "in_progress",
        TodoStatus::Completed => "completed",
        TodoStatus::Archived => "archived",
    });
    let priority_s = priority.map(|p| match p {
        TodoPriority::Low => "low",
        TodoPriority::Medium => "medium",
        TodoPriority::High => "high",
        TodoPriority::Urgent => "urgent",
    });
    // due_date: None = değiştirme, Some(None) = temizle, Some(Some) = ata.
    let (set_due, due_val) = match due_date {
        None => (false, None),
        Some(v) => (true, v),
    };
    // completed_at tek CASE'te: tamamlanınca şimdi, başka statüye
    // geçince NULL, statü değişmiyorsa dokunma.
    let completed_at: Option<DateTime<Utc>> = match status {
        Some(TodoStatus::Completed) => Some(Utc::now()),
        _ => None,
    };
    let touch_completed = status.is_some();
    let clear_completed = matches!(status, Some(s) if s != TodoStatus::Completed);
    Ok(sqlx::query_as::<_, Todo>(
        r#"UPDATE todos SET
             title = COALESCE($2, title),
             description = COALESCE($3, description),
             status = COALESCE($4::todo_status, status),
             priority = COALESCE($5::todo_priority, priority),
             due_date = CASE WHEN $6 THEN $7 ELSE due_date END,
             tags = COALESCE($8, tags),
             completed_at = CASE WHEN $11 THEN NULL WHEN $9 THEN $10 ELSE completed_at END
           WHERE id = $1 AND is_deleted = false
           RETURNING id, user_id, title, description,
                     status,
                     priority,
                     due_date, completed_at, tags, is_deleted, position, created_at, updated_at"#,
    )
    .bind(id)
    .bind(title)
    .bind(description)
    .bind(status_s)
    .bind(priority_s)
    .bind(set_due)
    .bind(due_val)
    .bind(tags)
    .bind(touch_completed && completed_at.is_some())
    .bind(completed_at)
    .bind(touch_completed && clear_completed)
    .fetch_one(db)
    .await?)
}

/// Soft delete.
pub async fn soft_delete(db: &PgPool, id: Uuid) -> Result<u64, AppError> {
    Ok(sqlx::query("UPDATE todos SET is_deleted = true WHERE id = $1 AND is_deleted = false")
        .bind(id)
        .execute(db)
        .await?
        .rows_affected())
}

/// Paylaşım kaydı oluşturur.
pub async fn share(
    db: &PgPool,
    todo_id: Uuid,
    user_id: Uuid,
    permission: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"INSERT INTO todo_shares (todo_id, shared_with_user_id, permission)
           VALUES ($1,$2,$3::share_permission)
           ON CONFLICT (todo_id, shared_with_user_id)
           DO UPDATE SET permission = EXCLUDED.permission"#,
    )
    .bind(todo_id)
    .bind(user_id)
    .bind(permission)
    .execute(db)
    .await?;
    Ok(())
}

/// Sürükle-bırak sıralaması: `ids` dizisindeki sıraya göre position ata.
/// Yalnızca sahibin todo'ları güncellenir (share'li kullanıcı sıralayamaz).
pub async fn reorder(db: &PgPool, user_id: Uuid, ids: &[Uuid]) -> Result<u64, AppError> {
    let mut n = 0u64;
    let mut tx = db.begin().await?;
    for (i, id) in ids.iter().enumerate() {
        let r = sqlx::query(
            "UPDATE todos SET position = $3 WHERE id = $1 AND user_id = $2 AND is_deleted = false",
        )
        .bind(id)
        .bind(user_id)
        .bind(i as i64)
        .execute(&mut *tx)
        .await?;
        n += r.rows_affected();
    }
    tx.commit().await?;
    Ok(n)
}

/// Kullanıcının todo'ya erişimi var mı (sahip veya paylaşılmış)?
pub async fn has_access(db: &PgPool, todo_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
    let n: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM todos t
           LEFT JOIN todo_shares s ON s.todo_id = t.id AND s.shared_with_user_id = $2
           WHERE t.id = $1 AND t.is_deleted = false AND (t.user_id = $2 OR s.id IS NOT NULL)"#,
    )
    .bind(todo_id)
    .bind(user_id)
    .fetch_one(db)
    .await?;
    Ok(n > 0)
}
