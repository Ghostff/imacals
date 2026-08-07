use actix_web::{HttpResponse, Responder};
use serde_json::json;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

pub async fn health_check() -> HttpResponse {
    JsonResponse::success(json!({
        "service": "imacals-api",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub async fn page_not_found() -> impl Responder {
    ErrorBag::NotFound("Page".to_string())
}