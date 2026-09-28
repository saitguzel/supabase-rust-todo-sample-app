//! Kullanıcı repository.
use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::profile::{Profile, Role};

/// Email ile auth kullanıcısını + profili getirir.
pub async fn find_auth_with_profile(
    db: &PgPool,
    email: &str,
) -> Result<Option<(Uuid, String, Profile)>, AppError> {
    let row: Option<(Uuid, String, Uuid, String, String, Option<String>, String, bool)> = sqlx::query_as(
        r#"SELECT u.id, u.encrypted_password, p.id, p.email, p.full_name, p.avatar_url, p.role::text, p.is_active
           FROM auth.users u JOIN profiles p ON p.id = u.id WHERE u.email = $1"#,
    )
    .bind(email.to_lowercase())
    .fetch_optional(db)
    .await?;
    Ok(row.map(|(uid, hash, pid, email, full_name, avatar, role_s, active)| {
        let profile = Profile {
            id: pid,
            email,
            full_name,
            avatar_url: avatar,
            role: Role::parse(&role_s).unwrap_or(Role::User),
            is_active: active,
            last_login_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        (uid, hash, profile)
    }))
}

/// Yeni kullanıcı oluşturur (auth.users + profiles, transaction).
pub async fn create_user(
    db: &PgPool,
    email: &str,
    password_hash: &str,
    full_name: &str,
) -> Result<Profile, AppError> {
    let mut tx = db.begin().await?;
    let email = email.to_lowercase();
    // id'yi açıkça üret: gerçek Supabase auth.users'ta id default'u yoktur
    // (Supabase Auth id'yi kendisi atar); yerel stub'da default olsa bile sorun olmaz.
    let uid = Uuid::new_v4();
    sqlx::query("INSERT INTO auth.users (id, email, encrypted_password) VALUES ($1,$2,$3)")
    .bind(uid)
    .bind(&email)
    .bind(password_hash)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        if let sqlx::Error::Database(d) = &e {
            if d.code().as_deref() == Some("23505") {
                return AppError::Conflict("email already registered".into());
            }
        }
        AppError::Db(e)
    })?;
    let profile = sqlx::query_as::<_, Profile>(
        r#"INSERT INTO profiles (id, email, full_name) VALUES ($1,$2,$3)
           RETURNING id, email, full_name, avatar_url,
                     role, is_active, last_login_at, created_at, updated_at"#,
    )
    .bind(uid)
    .bind(&email)
    .bind(full_name)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO notification_preferences (user_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(uid)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(profile)
}

/// Profil getirir.
pub async fn get_profile(db: &PgPool, id: Uuid) -> Result<Option<Profile>, AppError> {
    Ok(sqlx::query_as::<_, Profile>(
        r#"SELECT id, email, full_name, avatar_url, role,
                  is_active, last_login_at, created_at, updated_at
           FROM profiles WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?)
}

/// Son girişi günceller.
pub async fn touch_login(db: &PgPool, id: Uuid) -> Result<(), AppError> {
    sqlx::query("UPDATE profiles SET last_login_at = now() WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}
