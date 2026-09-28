//! AuditMiddleware — mutasyon isteklerini async olarak `audit_logs`'a yazar.
//!
//! Davranış (prompt § Logging):
//! - POST/PATCH/PUT/DELETE otomatik loglanır; GET yalnızca hassas path'lerde.
//! - `request_id` üretir, response header'a (`x-request-id`) ekler.
//! - Yazım asenkron (`tokio::spawn`) — request'i bloklamaz.
//! - Başarısız yetki denemeleri `severity='warn'` ile işaretlenmelidir
//!   (handler tarafında `log_denied` ile yapılır).
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use sqlx::PgPool;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::time::Instant;
use uuid::Uuid;

/// Audit middleware factory.
pub struct Audit {
    pub db: PgPool,
}

impl Audit {
    /// Yeni audit middleware.
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Audit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuditMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuditMiddleware {
            service: Rc::new(service),
            db: self.db.clone(),
        }))
    }
}

/// Gerçek middleware — her isteğe request_id ekler, mutasyonları loglar.
pub struct AuditMiddleware<S> {
    service: Rc<S>,
    db: PgPool,
}

impl<S, B> Service<ServiceRequest> for AuditMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let db = self.db.clone();
        Box::pin(async move {
            let request_id = Uuid::new_v4();
            let method = req.method().as_str().to_string();
            let path = req.path().to_string();
            let ip = req
                .connection_info()
                .realip_remote_addr()
                .unwrap_or("")
                .to_string();
            let ua = req
                .headers()
                .get("user-agent")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("")
                .to_string();
            // JWT'den kullanıcı çıkarmayı dene (yoksa anonim).
            let (user_id, actor_role) = extract_user(&req);
            let start = Instant::now();
            let res = service.call(req).await?;
            let status_code = res.status().as_u16() as i32;
            let duration_ms = start.elapsed().as_millis() as i32;

            // Hangi istekler loglanır?
            let is_mutation = matches!(method.as_str(), "POST" | "PATCH" | "PUT" | "DELETE");
            let is_sensitive_get =
                method == "GET" && (path.starts_with("/api/admin") || path.contains("/export"));
            if is_mutation || is_sensitive_get {
                let (action, entity) = action_entity(&method, &path);
                let status = if (200..400).contains(&status_code) {
                    "success"
                } else {
                    "failure"
                };
                let severity = if status == "failure" && res.status().as_u16() == 403 {
                    "warn"
                } else if status == "failure" {
                    "error"
                } else {
                    "info"
                };
                let db2 = db.clone();
                tokio::spawn(async move {
                    let _ = sqlx::query(
                        r#"INSERT INTO audit_logs
                        (user_id, actor_role, action, entity_type, entity_id, ip_address, user_agent, request_id, severity, status, duration_ms)
                        VALUES ($1,$2,$3,$4,$5, NULLIF($6,'')::inet, NULLIF($7,''), $8, $9::audit_severity, $10::audit_status, $11)"#,
                    )
                    .bind(user_id)
                    .bind(actor_role)
                    .bind(action)
                    .bind(entity)
                    .bind(path.clone())
                    .bind(ip)
                    .bind(ua)
                    .bind(request_id)
                    .bind(severity)
                    .bind(status)
                    .bind(duration_ms)
                    .execute(&db2)
                    .await;
                });
            }

            // Trace için response header.
            let mut res = res;
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-request-id"),
                request_id.to_string().parse().unwrap(),
            );
            Ok(res)
        })
    }
}

/// Authorization header'ından en iyi-efor kullanıcı çıkarımı.
fn extract_user(req: &ServiceRequest) -> (Option<Uuid>, String) {
    let Some(h) = req.headers().get("authorization").and_then(|v| v.to_str().ok()) else {
        return (None, "anonymous".into());
    };
    let Some(token) = h.strip_prefix("Bearer ") else {
        return (None, "anonymous".into());
    };
    // İmza doğrulamadan payload'u oku (audit için best-effort; yetki ayrıca doğrulanır).
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return (None, "anonymous".into());
    }
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    let Ok(bytes) = URL_SAFE_NO_PAD.decode(parts[1]) else {
        return (None, "anonymous".into());
    };
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return (None, "anonymous".into());
    };
    let id = v.get("sub").and_then(|s| s.as_str()).and_then(|s| s.parse().ok());
    let role = v
        .get("role")
        .and_then(|s| s.as_str())
        .unwrap_or("user")
        .to_string();
    (id, role)
}

/// (action, entity_type) çıkarımı — örn. POST /api/todos → (CREATE, todo).
fn action_entity(method: &str, path: &str) -> (String, String) {
    let entity = if path.contains("/todos") {
        "todo"
    } else if path.contains("/notifications") {
        "notification"
    } else if path.contains("/users") {
        "user"
    } else if path.contains("/logs") {
        "log"
    } else if path.contains("/settings") {
        "settings"
    } else if path.contains("/auth") {
        "session"
    } else {
        "unknown"
    };
    let action = match method {
        "POST" => {
            if path.contains("/login") {
                "LOGIN"
            } else if path.contains("/logout") {
                "LOGOUT"
            } else if path.contains("/export") {
                "EXPORT"
            } else {
                "CREATE"
            }
        }
        "PATCH" | "PUT" => {
            if path.contains("/role") {
                "ROLE_CHANGE"
            } else {
                "UPDATE"
            }
        }
        "DELETE" => "DELETE",
        _ => "READ",
    };
    (action.to_string(), entity.to_string())
}
