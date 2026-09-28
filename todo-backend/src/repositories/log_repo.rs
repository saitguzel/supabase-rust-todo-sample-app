//! Log repository — filtreli listeleme + export.
use sqlx::PgPool;

use crate::errors::AppError;
use crate::models::audit::AuditLog;

/// Filtreli audit log listesi.
#[allow(clippy::too_many_arguments)]
pub async fn list(
    db: &PgPool,
    action: Option<&str>,
    severity: Option<&str>,
    user_id: Option<&str>,
    entity_type: Option<&str>,
    from: Option<&str>,
    to: Option<&str>,
    search: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<AuditLog>, i64), AppError> {
    let rows = sqlx::query_as::<_, AuditLog>(
        r#"SELECT id, user_id, actor_role, action, entity_type, entity_id,
                  old_values, new_values,
                  ip_address::text AS ip_address,
                  user_agent, request_id,
                  severity::text AS severity, status::text AS status,
                  error_message, duration_ms, created_at
           FROM audit_logs
           WHERE ($1 IS NULL OR action = $1)
             AND ($2 IS NULL OR severity::text = $2)
             AND ($3 IS NULL OR user_id::text = $3)
             AND ($4 IS NULL OR entity_type = $4)
             AND ($5 IS NULL OR created_at >= $5::timestamptz)
             AND ($6 IS NULL OR created_at <= $6::timestamptz)
             AND ($7 IS NULL OR action ILIKE '%'||$7||'%' OR entity_id ILIKE '%'||$7||'%' OR COALESCE(error_message,'') ILIKE '%'||$7||'%')
           ORDER BY created_at DESC
           LIMIT $8 OFFSET $9"#,
    )
    .bind(action)
    .bind(severity)
    .bind(user_id)
    .bind(entity_type)
    .bind(from)
    .bind(to)
    .bind(search)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?;

    let total: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM audit_logs
           WHERE ($1 IS NULL OR action = $1)
             AND ($2 IS NULL OR severity::text = $2)
             AND ($5 IS NULL OR created_at >= $5::timestamptz)
             AND ($6 IS NULL OR created_at <= $6::timestamptz)"#,
    )
    .bind(action)
    .bind(severity)
    .bind(user_id)
    .bind(entity_type)
    .bind(from)
    .bind(to)
    .fetch_one(db)
    .await?;
    Ok((rows, total))
}
