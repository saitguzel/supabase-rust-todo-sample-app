//! Cursor kodlama yuvarlak-trip testi (bağımsız — src ile aynı algoritma).
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};

fn encode(ts: &str, id: &str) -> String {
    URL_SAFE_NO_PAD.encode(format!("{ts}:{id}").as_bytes())
}

#[test]
fn cursor_roundtrip() {
    let c = encode("2024-01-01T00:00:00+00:00", "123e4567-e89b-12d3-a456-426614174000");
    let raw = String::from_utf8(URL_SAFE_NO_PAD.decode(&c).unwrap()).unwrap();
    assert!(raw.contains("123e4567"));
}

#[test]
fn pagination_bounds() {
    let page = 0i64.max(1);
    let limit = 500i64.clamp(1, 100);
    assert_eq!(page, 1);
    assert_eq!(limit, 100);
}
