//! Admin DTO'ları (OpenAPI örnekleri dahil).
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, IntoParams)]
pub struct UserQuery {
    /// `user` | `moderator` | `admin` | `super_admin`
    pub role: Option<String>,
    pub active: Option<bool>,
    /// Email/ad üzerinde kısmi arama.
    pub search: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({"role": "moderator"}))]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1))]
    /// `user` | `moderator` | `admin` | `super_admin`
    pub role: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({"is_active": false}))]
pub struct UpdateStatusRequest {
    pub is_active: bool,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct LogQuery {
    /// Örnek: `LOGIN` | `CREATE` | `UPDATE` | `DELETE` | `ROLE_CHANGE` | `PERMISSION_DENIED`
    pub action: Option<String>,
    /// `debug` | `info` | `warn` | `error` | `critical`
    pub severity: Option<String>,
    pub user_id: Option<String>,
    /// Örnek: `todo` | `user` | `notification` | `session`
    pub entity_type: Option<String>,
    /// ISO-8601, örn. `2026-09-01T00:00:00Z`
    pub from: Option<String>,
    /// ISO-8601, örn. `2026-09-30T23:59:59Z`
    pub to: Option<String>,
    pub search: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct ExportQuery {
    /// `csv` | `json` (varsayılan json)
    pub format: Option<String>,
    pub action: Option<String>,
    pub severity: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "total_users": 6,
    "active_users": 6,
    "total_todos": 14,
    "completed_todos": 1,
    "overdue_todos": 0,
    "unread_notifications": 6,
    "audit_last_24h": 45
}))]
pub struct StatsResponse {
    pub total_users: i64,
    pub active_users: i64,
    pub total_todos: i64,
    pub completed_todos: i64,
    pub overdue_todos: i64,
    pub unread_notifications: i64,
    pub audit_last_24h: i64,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({"value": {"enabled": true}}))]
pub struct UpdateSettingsRequest {
    pub value: serde_json::Value,
}

/// GET /api/admin/users yanıtı (tip güvenli istemci için somut şema).
#[derive(Debug, Serialize, ToSchema)]
pub struct UserListResponse {
    pub data: Vec<crate::models::profile::Profile>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}

/// Rol/status mutasyon yanıtı.
#[derive(Debug, Serialize, ToSchema)]
pub struct MutationResponse {
    pub id: uuid::Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// PATCH /api/admin/notifications/broadcast yanıtı.
#[derive(Debug, Serialize, ToSchema)]
pub struct BroadcastResponse {
    pub sent: i64,
}

/// GET /api/admin/settings yanıtı (key/value çiftleri).
#[derive(Debug, Serialize, ToSchema)]
pub struct SettingEntry {
    pub key: String,
    pub value: serde_json::Value,
}

/// POST /api/todos/reorder & bulk yanıtı.
#[derive(Debug, Serialize, ToSchema)]
pub struct UpdatedCountResponse {
    pub updated: i64,
}

/// GET /api/notifications yanıtı.
#[derive(Debug, Serialize, ToSchema)]
pub struct NotificationListResponse {
    pub data: Vec<crate::models::notification::Notification>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub unread: i64,
}

/// GET /api/admin/logs yanıtı.
#[derive(Debug, Serialize, ToSchema)]
pub struct LogListResponse {
    pub data: Vec<crate::models::audit::AuditLog>,
    pub page: i64,
    pub limit: i64,
    pub total: i64,
}
