//! Sağlık kontrolleri: /health ve /health/db.
use actix_web::{get, web, HttpResponse};
use serde::Serialize;
use utoipa::ToSchema;

use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct Health {
    pub status: String,
    pub service: String,
    pub version: String,
}

/// Liveness — DB'ye dokunmaz.
#[utoipa::path(get, path = "/health", responses((status = 200, body = Health)))]
#[get("/health")]
pub async fn health() -> HttpResponse {
    HttpResponse::Ok().json(Health {
        status: "ok".into(),
        service: "todo-backend".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

/// Kök — tarayıcıda açıldığında servis bilgisi + bağlantılar döner.
#[utoipa::path(get, path = "/", responses((status = 200)))]
#[get("/")]
pub async fn index() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "service": "todo-backend",
        "version": env!("CARGO_PKG_VERSION"),
        "health": "/health",
        "db_health": "/health/db",
        "docs": "/swagger-ui/",
        "openapi": "/api-docs/openapi.json",
    }))
}

/// `/swagger-ui` (slash'sız) → `/swagger-ui/` kalıcı yönlendirme.
/// Tarayıcı adres çubuğuna slash'sız yazıldığında 404 yerine dokümana gider.
#[get("/swagger-ui")]
pub async fn swagger_redirect() -> HttpResponse {
    HttpResponse::Found()
        .insert_header((actix_web::http::header::LOCATION, "/swagger-ui/"))
        .finish()
}

/// Readiness — DB havuzunu yoklar.
#[utoipa::path(get, path = "/health/db", responses((status = 200), (status = 503)))]
#[get("/health/db")]
pub async fn health_db(state: web::Data<AppState>) -> HttpResponse {
    match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "ok", "db": "up"})),
        Err(e) => {
            tracing::error!(%e, "db health failed");
            HttpResponse::ServiceUnavailable().json(serde_json::json!({"status": "down"}))
        }
    }
}
