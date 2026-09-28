//! RFC 7807 Problem Details formatında merkezi hata yönetimi.
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use uuid::Uuid;

/// API hata gövdesi (RFC 7807).
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    pub type_: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<Uuid>,
}

/// Uygulama hatası — tüm servis/handler katmanları bunu kullanır.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation: {0}")]
    Validation(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    /// Henüz tel kilitlenmedi — governor middleware döndürür; API bütünlüğü için tutulur.
    #[allow(dead_code)]
    #[error("Rate limited")]
    RateLimited,
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Db(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        self.status()
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status();
        // Detayda içsel DB hatasını sızdırma.
        let detail = match self {
            Self::Db(e) => {
                tracing::error!(error = %e, "db error");
                "Internal server error".to_string()
            }
            Self::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                "Internal server error".to_string()
            }
            other => other.to_string(),
        };
        HttpResponse::build(status).json(ProblemDetails {
            type_: format!("https://api.todo.local/problems/{}", status.as_u16()),
            title: status.canonical_reason().unwrap_or("Error").to_string(),
            status: status.as_u16(),
            detail,
            instance: None,
            request_id: None,
        })
    }
}

/// Validator hatalarını AppError'a çevirir.
pub fn validation_error(e: validator::ValidationErrors) -> AppError {
    AppError::Validation(e.to_string())
}
