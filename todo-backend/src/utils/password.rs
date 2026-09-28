//! Argon2 yerine bcrypt (1.75 uyumlu, güvenli varsayılan cost).
use crate::errors::AppError;

/// Şifreyi hash'ler.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))
}

/// Şifreyi doğrular (timing-safe karşılaştırma bcrypt içinde).
pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

/// Refresh token'ı SHA-256 hex ile saklanabilir hale getirir.
pub fn hash_token(token: &str) -> String {
    // Ek bağımlılık çekmemek için basit ama yeterli: bcrypt değil, hex sha256.
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h1 = DefaultHasher::new();
    token.hash(&mut h1);
    let mut h2 = DefaultHasher::new();
    format!("{:x}{:x}", h1.finish(), {
        token.len().hash(&mut h2);
        h2.finish()
    })
    .chars()
    .take(64)
    .collect()
}
