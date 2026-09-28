//! Auth DTO'ları (OpenAPI örnekleri dahil).
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "email": "yeni@ornek.com",
    "password": "parola1234",
    "full_name": "Ayşe Yılmaz"
}))]
pub struct RegisterRequest {
    #[validate(email, length(max = 255))]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    #[validate(length(min = 1, max = 120))]
    pub full_name: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "email": "admin@demo.local",
    "password": "demo1234"
}))]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(example = json!({
    "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}))]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "access_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "refresh_token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
    "token_type": "Bearer",
    "expires_in": 3600
}))]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
    "email": "admin@demo.local",
    "full_name": "Demo Admin",
    "role": "admin"
}))]
pub struct MeResponse {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub role: String,
}
