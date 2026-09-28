//! Todo Backend — Actix Web 4 + Supabase Postgres.
//!
//! Çalıştırma:
//! ```sh
//! cp .env.example .env   # DATABASE_URL ve JWT secret'ı doldur
//! docker compose up --build
//! ```
//! Sağlık: `GET /health`, DB: `GET /health/db`, Doküman: `GET /swagger-ui/`.
#![deny(warnings)]

mod config;
mod dto;
mod errors;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;
mod state;
mod utils;

use std::sync::Arc;

use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{http, middleware::Logger, web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::AppConfig;
use crate::middleware::audit::Audit;
use crate::state::AppState;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // 1) Yapılandırma + loglama.
    let config = AppConfig::from_env()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
    tracing::info!(app = "todo-backend", version = env!("CARGO_PKG_VERSION"), "starting");

    // 2) DB havuzu + migration.
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("migrations applied");

    // Demo hesapları (idempotent; SEED_DEMO_USERS=false ile kapatılır).
    match crate::services::seed_service::seed_demo_users(&pool, &config).await {
        Ok(n) if n > 0 => tracing::info!(count = n, "demo users seeded"),
        Ok(_) => {}
        Err(e) => tracing::warn!(%e, "demo seed failed (non-fatal)"),
    }

    let state = web::Data::new(AppState {
        db: pool.clone(),
        config: Arc::new(config.clone()),
    });

    // 3) Arka plan worker — due-date kontrolü (tokio-cron-scheduler).
    spawn_due_worker(pool.clone(), config.clone());

    // 4) Rate limiting (IP bazlı, governor).
    // DİKKAT: governor'da `per_second(n)` "n saniyede 1 token yenile" demektir
    // ("saniyede n istek" değil). Saniyede `rate_per_second` istek istiyorsak
    // period = 1000/rate ms olmalı.
    let refill_ms = (1000 / config.rate_per_second.max(1)).max(1);
    let governor_conf = GovernorConfigBuilder::default()
        .per_millisecond(refill_ms)
        .burst_size(config.rate_burst)
        .finish()
        .expect("governor config");
    let bind = (config.host.clone(), config.port);

    // 5) HTTP sunucusu + graceful shutdown.
    let server = HttpServer::new(move || {
        let cors = build_cors(&config);
        App::new()
            .app_data(state.clone())
            .app_data(web::JsonConfig::default().limit(256 * 1024))
            .app_data(web::PayloadConfig::new(256 * 1024))
            .wrap(tracing_actix_web::TracingLogger::default())
            .wrap(Logger::default())
            .wrap(Audit::new(pool.clone()))
            .wrap(cors)
            .wrap(Governor::new(&governor_conf))
            .configure(routes::configure)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", routes::ApiDoc::openapi()),
            )
            // Güvenlik başlıkları (Helmet benzeri minimal set).
            .wrap(actix_web::middleware::DefaultHeaders::new()
                .add(("X-Content-Type-Options", "nosniff"))
                .add(("X-Frame-Options", "DENY"))
                .add(("Referrer-Policy", "no-referrer"))
                .add(("Permissions-Policy", "camera=(), microphone=(), geolocation=()")))
    })
    .bind(&bind)?
    .run();

    tracing::info!(host = %bind.0, port = bind.1, "listening");
    // SIGTERM/SIGINT ile graceful shutdown (actix handle_signals varsayılanı).
    server.await?;
    tracing::info!("shutdown complete");
    Ok(())
}

/// CORS — whitelist env'den gelir.
fn build_cors(config: &AppConfig) -> Cors {
    let mut cors = Cors::default()
        .allowed_methods(vec!["GET", "POST", "PATCH", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![http::header::AUTHORIZATION, http::header::CONTENT_TYPE])
        .expose_headers(vec!["x-request-id"])
        .max_age(3600);
    for origin in &config.cors_origins {
        cors = cors.allowed_origin(origin.as_str());
    }
    cors
}

/// Due-date worker — cron takvimine göre `check_due_todos` çalıştırır.
fn spawn_due_worker(pool: sqlx::PgPool, config: AppConfig) {
    tokio::spawn(async move {
        use tokio_cron_scheduler::{Job, JobScheduler};
        let sched = match JobScheduler::new().await {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(%e, "scheduler init failed");
                return;
            }
        };
        let db = pool.clone();
        let hours = config.due_soon_hours;
        let cron = config.check_due_cron.clone();
        let job = match Job::new_async(cron.as_str(), move |_, _| {
            let db = db.clone();
            Box::pin(async move {
                if let Err(e) = crate::services::notification_service::check_due_todos(&db, hours).await {
                    tracing::error!(%e, "due check failed");
                }
            })
        }) {
            Ok(j) => j,
            Err(e) => {
                tracing::error!(%e, "cron job invalid, worker disabled");
                return;
            }
        };
        if let Err(e) = sched.add(job).await {
            tracing::error!(%e, "scheduler add failed");
            return;
        }
        if let Err(e) = sched.start().await {
            tracing::error!(%e, "scheduler start failed");
        }
    });
}
