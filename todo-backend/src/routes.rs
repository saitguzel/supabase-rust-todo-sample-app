//! Route tanımları + OpenAPI (Swagger UI `/swagger-ui/` adresinde).
use actix_web::web;
use utoipa::OpenApi;

use crate::handlers::{admin, auth, health, notifications, todos};

/// OpenAPI dokümanı (Swagger UI `/swagger-ui/` adresinde).
#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handlers::health::index,
        crate::handlers::health::health,
        crate::handlers::health::health_db,
        // Auth
        crate::handlers::auth::register,
        crate::handlers::auth::login,
        crate::handlers::auth::refresh,
        crate::handlers::auth::logout,
        crate::handlers::auth::me,
        // Todos
        crate::handlers::todos::list,
        crate::handlers::todos::create,
        crate::handlers::todos::get_one,
        crate::handlers::todos::update,
        crate::handlers::todos::delete,
        crate::handlers::todos::complete,
        crate::handlers::todos::share,
        crate::handlers::todos::bulk_alt,
        crate::handlers::todos::reorder,
        // Notifications
        crate::handlers::notifications::list,
        crate::handlers::notifications::mark_read,
        crate::handlers::notifications::read_all,
        crate::handlers::notifications::delete,
        // Admin
        crate::handlers::admin::users::list_users,
        crate::handlers::admin::users::update_role,
        crate::handlers::admin::users::update_status,
        crate::handlers::admin::logs::list_logs,
        crate::handlers::admin::logs::export_logs,
        crate::handlers::admin::stats::stats,
        crate::handlers::admin::stats::broadcast,
        crate::handlers::admin::stats::get_settings,
        crate::handlers::admin::stats::update_settings,
    ),
    components(schemas(
        crate::handlers::health::Health,
        crate::dto::todo::PaginatedTodo,
        crate::dto::auth::RegisterRequest,
        crate::dto::auth::LoginRequest,
        crate::dto::auth::RefreshRequest,
        crate::dto::auth::TokenResponse,
        crate::dto::auth::MeResponse,
        crate::dto::todo::CreateTodoRequest,
        crate::dto::todo::UpdateTodoRequest,
        crate::dto::todo::ShareTodoRequest,
        crate::dto::todo::BulkTodoRequest,
        crate::dto::todo::ReorderRequest,
        crate::dto::notification::BroadcastRequest,
        crate::dto::notification::UnreadCount,
        crate::dto::admin::UpdateRoleRequest,
        crate::dto::admin::UpdateStatusRequest,
        crate::dto::admin::StatsResponse,
        crate::dto::admin::UpdateSettingsRequest,
        crate::dto::admin::UserListResponse,
        crate::dto::admin::MutationResponse,
        crate::dto::admin::BroadcastResponse,
        crate::dto::admin::SettingEntry,
        crate::dto::admin::UpdatedCountResponse,
        crate::dto::admin::NotificationListResponse,
        crate::dto::admin::LogListResponse,
        crate::models::todo::Todo,
        crate::models::todo::TodoStatus,
        crate::models::todo::TodoPriority,
        crate::models::profile::Profile,
        crate::models::profile::Role,
        crate::models::notification::Notification,
        crate::models::notification::NotificationType,
        crate::models::audit::AuditLog,
        crate::errors::ProblemDetails,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "health", description = "Sağlık kontrolleri"),
        (name = "auth", description = "Kimlik doğrulama"),
        (name = "todos", description = "Todo CRUD"),
        (name = "notifications", description = "Bildirimler"),
        (name = "admin", description = "Yönetim (rol korumalı)"),
    )
)]
pub struct ApiDoc;

/// Swagger UI'da "Authorize" butonunu etkinleştiren Bearer security scheme'i.
struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer",
            utoipa::openapi::security::SecurityScheme::Http(
                utoipa::openapi::security::HttpBuilder::new()
                    .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

/// Tüm API route'larını kaydeder.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(health::index)
        .service(health::swagger_redirect)
        .service(health::health)
        .service(health::health_db)
        .service(
            web::scope("/api")
                .service(auth::scope())
                .service(todos::scope())
                .service(notifications::scope())
                .service(
                    web::scope("/admin")
                        .service(admin::users::list_users)
                        .service(admin::users::update_role)
                        .service(admin::users::update_status)
                        .service(admin::logs::list_logs)
                        .service(admin::logs::export_logs)
                        .service(admin::stats::stats)
                        .service(admin::stats::broadcast)
                        .service(admin::stats::get_settings)
                        .service(admin::stats::update_settings),
                ),
        );
}
