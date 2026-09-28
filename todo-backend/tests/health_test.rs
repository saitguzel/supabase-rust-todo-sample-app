//! Sağlık endpoint'i entegrasyon testi.
use actix_web::{test, App};

#[actix_web::test]
async fn health_ok() {
    let app = test::init_service(App::new().service(todo_backend_health_probe)).await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

use actix_web::get;
use actix_web::HttpResponse;

#[get("/health")]
async fn todo_backend_health_probe() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}
