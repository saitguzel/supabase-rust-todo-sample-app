//! Audit servis — manuel log yazımı (middleware otomatik yazar; bu hassas olaylar için).
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;

/// Kritik olayı audit_logs'a yazar (best-effort, hata yutmaz — caller'a döner).
#[allow(clippy::too_many_arguments)]
pub async fn log_event(
    db: &PgPool,
    user_id: Option<Uuid>,
    actor_role: &str,
    action: &str,
    entity_type: &str,
    entity_id: &str,
    old_values: Option<serde_json::Value>,
    new_values: Option<serde_json::Value>,
    request_id: Option<Uuid>,
    severity: &str,
    status: &str,
) -> Result<(), AppError> {
    sqlx::query(
        r#"INSERT INTO audit_logs
           (user_id, actor_role, action, entity_type, entity_id, old_values, new_values, request_id, severity, status)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9::audit_severity,$10::audit_status)"#,
    )
    .bind(user_id)
    .bind(actor_role)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(old_values)
    .bind(new_values)
    .bind(request_id)
    .bind(severity)
    .bind(status)
    .execute(db)
    .await?;
    Ok(())
}

/// Yetkisiz erişim denemesini warn olarak loglar (async, bloklamaz).
pub fn log_denied(
    db: PgPool,
    user_id: Option<Uuid>,
    actor_role: String,
    entity_type: String,
    entity_id: String,
) {
    tokio::spawn(async move {
        let _ = sqlx::query(
            "INSERT INTO audit_logs (user_id, actor_role, action, entity_type, entity_id, severity, status, error_message) VALUES ($1,$2,'PERMISSION_DENIED',$3,$4,'warn','failure','forbidden')",
        )
        .bind(user_id)
        .bind(actor_role)
        .bind(entity_type)
        .bind(entity_id)
        .execute(&db)
        .await;
    });
}
