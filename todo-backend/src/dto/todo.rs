//! Todo DTO'ları (OpenAPI örnekleri dahil).
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

use crate::models::todo::{Todo, TodoPriority, TodoStatus};

#[derive(Debug, Deserialize, IntoParams)]
pub struct TodoQuery {
    /// Örnek: `pending` | `in_progress` | `completed` | `archived`
    pub status: Option<String>,
    /// Örnek: `low` | `medium` | `high` | `urgent`
    pub priority: Option<String>,
    /// Başlık/açıklamada kısmi arama.
    pub search: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    /// Cursor (keyset pagination — created_at + id, opak base64).
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "title": "Raporu hazırla",
    "description": "Q3 satış raporunu tamamla",
    "priority": "high",
    "due_date": "2026-10-01T17:00:00Z",
    "tags": ["is", "rapor"]
}))]
pub struct CreateTodoRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(max = 5000))]
    pub description: Option<String>,
    pub status: Option<TodoStatus>,
    pub priority: Option<TodoPriority>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "title": "Raporu hazırla (güncel)",
    "status": "in_progress",
    "priority": "urgent"
}))]
pub struct UpdateTodoRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: Option<String>,
    #[validate(length(max = 5000))]
    pub description: Option<String>,
    pub status: Option<TodoStatus>,
    pub priority: Option<TodoPriority>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "email": "user@demo.local",
    "permission": "read"
}))]
pub struct ShareTodoRequest {
    #[validate(email)]
    pub email: String,
    /// `read` (varsayılan) veya `write`
    pub permission: Option<String>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "ids": ["3fa85f64-5717-4562-b3fc-2c963f66afa6"],
    "action": "complete"
}))]
pub struct BulkTodoRequest {
    #[validate(length(min = 1, max = 100))]
    pub ids: Vec<uuid::Uuid>,
    /// `complete` | `archive` | `delete`
    pub action: String,
}

/// Sürükle-bırak sıralama: `ids` dizisi hedef sırayı verir (0,1,2…).
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "ids": [
        "3fa85f64-5717-4562-b3fc-2c963f66afa6",
        "9b2f1cde-1111-4c1c-9f5a-2c963f66afa7"
    ]
}))]
pub struct ReorderRequest {
    #[validate(length(min = 1, max = 500))]
    pub ids: Vec<uuid::Uuid>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Paginated<T: Serialize> {
    /// Örnek: `[{"id": "...", "title": "Raporu hazırla", "status": "pending", ...}]`
    pub data: Vec<T>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    /// Varsa sonraki sayfa için opak cursor.
    #[schema(example = "MjAyNi0wOS0yNVQxMjowMDAwWrY3")]
    pub next_cursor: Option<String>,
}

/// OpenAPI için somutlaştırılmış jenerik — utoipa generic $ref'lerini çözemesin diye.
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "data": [{"id": "3fa85f64-5717-4562-b3fc-2c963f66afa6", "title": "Raporu hazırla", "status": "pending", "priority": "high"}],
    "page": 1,
    "limit": 20,
    "total": 1,
    "next_cursor": null
}))]
pub struct PaginatedTodo {
    pub data: Vec<Todo>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub next_cursor: Option<String>,
}
