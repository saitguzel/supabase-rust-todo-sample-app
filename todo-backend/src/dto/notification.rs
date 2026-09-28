//! Notification DTO'ları (OpenAPI örnekleri dahil).
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Deserialize, IntoParams)]
pub struct NotificationQuery {
    /// `true` → yalnızca okunmamışlar.
    pub unread: Option<bool>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[schema(example = json!({
    "title": "Bakım duyurusu",
    "message": "Sistem bu gece 02:00-03:00 bakımda olacak."
}))]
pub struct BroadcastRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(length(max = 2000))]
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({"unread": 3}))]
pub struct UnreadCount {
    pub unread: i64,
}
