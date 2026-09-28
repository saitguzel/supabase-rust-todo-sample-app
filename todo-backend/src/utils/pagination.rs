//! Cursor (keyset) sayfalama yardımcısı.
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Cursor içeriği: (created_at, id).
#[derive(Debug, Clone)]
pub struct Cursor {
    pub created_at: DateTime<Utc>,
    pub id: Uuid,
}

/// Cursor'ı opak string'e kodlar.
pub fn encode_cursor(created_at: &DateTime<Utc>, id: &Uuid) -> String {
    let raw = format!("{}:{}", created_at.to_rfc3339(), id);
    URL_SAFE_NO_PAD.encode(raw.as_bytes())
}

/// Cursor'ı çözer; bozuksa None döner.
pub fn decode_cursor(s: &str) -> Option<Cursor> {
    let bytes = URL_SAFE_NO_PAD.decode(s).ok()?;
    let raw = String::from_utf8(bytes).ok()?;
    let (ts, id) = raw.rsplit_once(':')?;
    Some(Cursor {
        created_at: ts.parse().ok()?,
        id: id.parse().ok()?,
    })
}
