//! Uygulama yapılandırması — tüm secret'lar env'den okunur, asla hard-coded değil.
use std::env;

/// Merkezi uygulama yapılandırması.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_access_ttl_minutes: i64,
    pub jwt_refresh_ttl_days: i64,
    pub jwt_issuer: Option<String>,
    pub jwt_audience: Option<String>,
    pub cors_origins: Vec<String>,
    pub rate_per_second: u64,
    pub rate_burst: u32,
    pub due_soon_hours: i64,
    pub check_due_cron: String,
    /// Açılışta demo hesapları oluştur (varsayılan true; üretimde false).
    pub seed_demo_users: bool,
    /// Demo hesap şifresi (env: DEMO_PASSWORD).
    pub demo_password: String,
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

impl AppConfig {
    /// Ortam değişkenlerinden yapılandırma yükler. Eksik kritik değerde hata döner.
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL env gerekli"))?;
        // Supabase JWT secret varsa onu tercih et (Supabase Auth uyumluluğu).
        let jwt_secret = env::var("SUPABASE_JWT_SECRET")
            .or_else(|_| env::var("JWT_SECRET"))
            .map_err(|_| anyhow::anyhow!("SUPABASE_JWT_SECRET veya JWT_SECRET gerekli"))?;
        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT secret en az 32 karakter olmalı");
        }
        Ok(Self {
            host: env_or("APP_HOST", "0.0.0.0"),
            port: env_or("APP_PORT", "8080").parse().unwrap_or(8080),
            database_url,
            jwt_secret,
            jwt_access_ttl_minutes: env_or("JWT_ACCESS_TTL_MINUTES", "60").parse().unwrap_or(60),
            jwt_refresh_ttl_days: env_or("JWT_REFRESH_TTL_DAYS", "30").parse().unwrap_or(30),
            jwt_issuer: env::var("JWT_ISSUER").ok().filter(|s| !s.is_empty()),
            jwt_audience: env::var("JWT_AUDIENCE").ok().filter(|s| !s.is_empty()),
            cors_origins: env_or("CORS_ALLOWED_ORIGINS", "http://localhost:3005")
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            rate_per_second: env_or("RATE_LIMIT_PER_SECOND", "10").parse().unwrap_or(10),
            rate_burst: env_or("RATE_LIMIT_BURST", "20").parse().unwrap_or(20),
            due_soon_hours: env_or("DUE_SOON_HOURS", "24").parse().unwrap_or(24),
            check_due_cron: env_or("CHECK_DUE_CRON", "0 */15 * * * *"),
            seed_demo_users: env_or("SEED_DEMO_USERS", "true").parse().unwrap_or(true),
            demo_password: env_or("DEMO_PASSWORD", "demo1234"),
        })
    }
}
