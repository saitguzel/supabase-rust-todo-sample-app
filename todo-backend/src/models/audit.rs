//! Audit log entity.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// audit_logs satırı.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
pub struct AuditLog {
    pub id: i64,
    pub user_id: Option<Uuid>,
    pub actor_role: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: String,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<Uuid>,
    pub severity: String,
    pub status: String,
    pub error_message: Option<String>,
    pub duration_ms: i32,
    pub created_at: DateTime<Utc>,
}
