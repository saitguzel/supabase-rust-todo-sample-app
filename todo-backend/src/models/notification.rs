//! Notification entity.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, utoipa::ToSchema)]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    TodoAssigned,
    TodoDueSoon,
    TodoOverdue,
    TodoCompleted,
    SystemAnnouncement,
    RoleChanged,
}

/// notifications satırı.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub r#type: NotificationType,
    pub title: String,
    pub message: String,
    pub payload: serde_json::Value,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
