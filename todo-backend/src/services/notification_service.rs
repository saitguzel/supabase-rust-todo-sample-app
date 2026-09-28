//! Bildirim servisi — trait ile soyutlanmış + cron worker.
//!
//! Kanallar: 1) In-app (notifications tablosu + Supabase Realtime uyumlu),
//! 2) Email (soyut — tak-çıkar), 3) Web Push (soyut).
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::notification::NotificationType;
use crate::repositories::notification_repo;
use crate::services::todo_service;

/// Bildirim kanalı arayüzü — tak-çıkar kanal implementasyonları için.
/// Doğrudan DB yazan yollarda kullanılmaz; dış kanal entegrasyonları (Resend,
/// Edge Function, Web Push) bu trait'i implemente eder.
#[allow(dead_code)]
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Bildirimi kanala gönderir.
    async fn send(&self, user_id: Uuid, title: &str, message: &str) -> Result<(), AppError>;
}

/// In-app kanal — DB'ye yazar (Supabase Realtime bu tabloyu dinler).
#[allow(dead_code)]
pub struct InAppChannel {
    pub db: PgPool,
}

#[async_trait]
impl NotificationChannel for InAppChannel {
    async fn send(&self, user_id: Uuid, title: &str, message: &str) -> Result<(), AppError> {
        notification_repo::create(
            &self.db,
            user_id,
            NotificationType::SystemAnnouncement,
            title,
            message,
            &serde_json::json!({}),
        )
        .await?;
        Ok(())
    }
}

/// Email kanalı — şimdilik log'a yazar (Resend/Edge Function tak-çıkar).
#[allow(dead_code)]
pub struct EmailChannel;

#[async_trait]
impl NotificationChannel for EmailChannel {
    async fn send(&self, user_id: Uuid, title: &str, message: &str) -> Result<(), AppError> {
        tracing::info!(%user_id, title, message, channel = "email", "notification queued");
        Ok(())
    }
}

/// Due-date kontrolü — cron job'dan çağrılır.
/// Vadesi `due_soon_hours` içinde olan veya geçmiş todo'lar için bildirim üretir.
pub async fn check_due_todos(db: &PgPool, due_soon_hours: i64) -> Result<u64, AppError> {
    let due = todo_service::find_due(db, due_soon_hours).await?;
    let mut n = 0u64;
    for (todo_id, user_id, title, due_date) in due {
        let overdue = due_date.map(|d| d < chrono::Utc::now()).unwrap_or(false);
        let (ntype, msg) = if overdue {
            (NotificationType::TodoOverdue, format!("'{title}' gecikti!"))
        } else {
            (
                NotificationType::TodoDueSoon,
                format!("'{title}' yakında vadesi doluyor."),
            )
        };
        // Aynı todo için son 24 saatte bildirim varsa tekrarlama.
        let recent: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND payload->>'todo_id' = $2 AND created_at > now() - interval '24 hours'",
        )
        .bind(user_id)
        .bind(todo_id.to_string())
        .fetch_one(db)
        .await?;
        if recent > 0 {
            continue;
        }
        // Kullanıcı tercihi kapalıysa atla.
        let prefs: Option<(bool, bool)> = sqlx::query_as(
            "SELECT COALESCE(due_soon, true), COALESCE(overdue, true) FROM notification_preferences WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_optional(db)
        .await?;
        if let Some((due_soon, overdue_pref)) = prefs {
            if (overdue && !overdue_pref) || (!overdue && !due_soon) {
                continue;
            }
        }
        notification_repo::create(
            db,
            user_id,
            ntype,
            if overdue { "Todo gecikti" } else { "Todo vadesi yaklaşıyor" },
            &msg,
            &serde_json::json!({"todo_id": todo_id}),
        )
        .await?;
        n += 1;
    }
    if n > 0 {
        tracing::info!(count = n, "due notifications created");
    }
    Ok(n)
}

/// Admin broadcast — tüm aktif kullanıcılara duyuru.
pub async fn broadcast(db: &PgPool, title: &str, message: &str) -> Result<u64, AppError> {
    let ids: Vec<Uuid> =
        sqlx::query_scalar("SELECT id FROM profiles WHERE is_active = true")
            .fetch_all(db)
            .await?;
    let mut n = 0u64;
    for id in ids {
        notification_repo::create(
            db,
            id,
            NotificationType::SystemAnnouncement,
            title,
            message,
            &serde_json::json!({"broadcast": true}),
        )
        .await?;
        n += 1;
    }
    Ok(n)
}
