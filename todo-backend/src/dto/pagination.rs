//! Sayfalama yardımcısı (limit 1-100, cursor keyset).
use serde::Deserialize;

/// Sayfalama parametrelerini normalize eder.
#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub page: i64,
    pub limit: i64,
}

impl Pagination {
    /// page/limit değerlerini güvenli aralığa çeker.
    pub fn new(page: Option<i64>, limit: Option<i64>) -> Self {
        let page = page.unwrap_or(1).max(1);
        let limit = limit.unwrap_or(20).clamp(1, 100);
        Self { page, limit }
    }

    /// SQL OFFSET hesaplar.
    pub fn offset(&self) -> i64 {
        (self.page - 1) * self.limit
    }
}
