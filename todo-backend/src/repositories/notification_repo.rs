//! Notification repository.
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::notification::{Notification, NotificationType};

fn type_str(t: NotificationType) -> &'static str {
    match t {
        NotificationType::TodoAssigned => "todo_assigned",
        NotificationType::TodoDueSoon => "todo_due_soon",
        NotificationType::TodoOverdue => "todo_overdue",
        NotificationType::TodoCompleted => "todo_completed",
        NotificationType::SystemAnnouncement => "system_announcement",
        NotificationType::RoleChanged => "role_changed",
    }
}

/// Bildirim oluşturur.
pub async fn create(
    db: &PgPool,
    user_id: Uuid,
    ntype: NotificationType,
    title: &str,
    message: &str,
    payload: &serde_json::Value,
) -> Result<Notification, AppError> {
    Ok(sqlx::query_as::<_, Notification>(
        r#"INSERT INTO notifications (user_id, type, title, message, payload)
           VALUES ($1, $2::notification_type, $3, $4, $5)
           RETURNING id, user_id, "type",
                     title, message, payload, is_read, read_at, created_at"#,
    )
    .bind(user_id)
    .bind(type_str(ntype))
    .bind(title)
    .bind(message)
    .bind(payload)
    .fetch_one(db)
    .await?)
}

/// Kullanıcının bildirimleri.
pub async fn list(
    db: &PgPool,
    user_id: Uuid,
    unread_only: bool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<Notification>, i64), AppError> {
    let rows = sqlx::query_as::<_, Notification>(
        r#"SELECT id, user_id, "type",
                  title, message, payload, is_read, read_at, created_at
           FROM notifications
           WHERE user_id = $1 AND (NOT $2 OR is_read = false)
           ORDER BY created_at DESC LIMIT $3 OFFSET $4"#,
    )
    .bind(user_id)
    .bind(unread_only)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?;
    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND (NOT $2 OR is_read = false)",
    )
    .bind(user_id)
    .bind(unread_only)
    .fetch_one(db)
    .await?;
    Ok((rows, total))
}

/// Okunmadı sayısı.
pub async fn unread_count(db: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
    Ok(
        sqlx::query_scalar("SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = false")
            .bind(user_id)
            .fetch_one(db)
            .await?,
    )
}
