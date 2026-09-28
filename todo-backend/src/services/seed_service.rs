//! Demo kullanıcı seed'i — geliştirme/demo ortamı için.
//!
//! `SEED_DEMO_USERS=true` (varsayılan) iken uygulama açılışında idempotent
//! şekilde üç demo hesap oluşturur: admin / moderator / user.
//! Üretimde `SEED_DEMO_USERS=false` yapın.
use sqlx::PgPool;

use crate::config::AppConfig;
use crate::errors::AppError;
use crate::repositories::user_repo;
use crate::utils::password::hash_password;

/// Demo hesap tanımı.
struct DemoUser {
    email: &'static str,
    full_name: &'static str,
    role: &'static str,
}

const DEMO_USERS: [DemoUser; 3] = [
    DemoUser { email: "admin@demo.local", full_name: "Demo Admin", role: "admin" },
    DemoUser { email: "mod@demo.local", full_name: "Demo Moderator", role: "moderator" },
    DemoUser { email: "user@demo.local", full_name: "Demo User", role: "user" },
];

/// Demo kullanıcı şifresi (env ile değiştirilebilir; en az 8 karakter olmalı).
pub fn demo_password(cfg: &AppConfig) -> String {
    let p = cfg.demo_password.clone();
    if p.len() >= 8 {
        p
    } else {
        "demo1234".to_string()
    }
}

/// Demo hesapları oluşturur; zaten var olanlar atlanır (idempotent).
pub async fn seed_demo_users(db: &PgPool, cfg: &AppConfig) -> Result<usize, AppError> {
    if !cfg.seed_demo_users {
        return Ok(0);
    }
    let password = demo_password(cfg);
    let hash = hash_password(&password)?;
    let mut created = 0usize;
    for u in DEMO_USERS {
        let exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM profiles WHERE email = $1")
            .bind(u.email)
            .fetch_optional(db)
            .await?;
        if exists.is_some() {
            continue;
        }
        match user_repo::create_user(db, u.email, &hash, u.full_name).await {
            Ok(mut profile) => {
                // create_user her zaman 'user' rolüyle açar; demo rolünü ata.
                sqlx::query("UPDATE profiles SET role = $2::user_role WHERE id = $1")
                    .bind(profile.id)
                    .bind(u.role)
                    .execute(db)
                    .await?;
                profile.role = crate::models::profile::Role::parse(u.role)
                    .unwrap_or(crate::models::profile::Role::User);
                tracing::info!(email = u.email, role = u.role, "demo user seeded");
                created += 1;
            }
            // Paralel seed / tekrarlı açılışta çakışma — sessizce atla.
            Err(AppError::Conflict(_)) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(created)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_password_fallback() {
        let mut cfg = AppConfig {
            host: "0.0.0.0".into(),
            port: 8080,
            database_url: "postgres://x".into(),
            jwt_secret: "s".repeat(32),
            jwt_access_ttl_minutes: 60,
            jwt_refresh_ttl_days: 30,
            jwt_issuer: None,
            jwt_audience: None,
            cors_origins: vec![],
            rate_per_second: 10,
            rate_burst: 20,
            due_soon_hours: 24,
            check_due_cron: "0 */15 * * * *".into(),
            seed_demo_users: true,
            demo_password: "short".into(),
        };
        assert_eq!(demo_password(&cfg), "demo1234");
        cfg.demo_password = "super-secret-9".into();
        assert_eq!(demo_password(&cfg), "super-secret-9");
    }
}
