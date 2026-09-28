//! Paylaşılan uygulama durumu (DB havuzu + yapılandırma).
use sqlx::PgPool;
use std::sync::Arc;

use crate::config::AppConfig;

/// Tüm handler'lara enjekte edilen durum.
#[derive(Clone)]
pub struct AppState {
    /// Postgres bağlantı havuzu (Supabase Postgres ile uyumlu).
    pub db: PgPool,
    /// Ortam yapılandırması.
    pub config: Arc<AppConfig>,
}
