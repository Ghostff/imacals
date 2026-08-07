use actix_web::HttpResponse;
use serde::Serialize;
use serde_json::json;

use crate::utilities::error_bag::ErrorBag;

/// JsonResponse centralizes common JSON HTTP responses for controllers.
///
/// Why this exists (AI-friendly summary):
/// - Keeps controllers thin and consistent across the codebase.
/// - Provides a single place to tweak response envelope (status/message/data) and logging.
/// - Each helper returns an `HttpResponse` ready to be returned from handlers.
///
/// Usage examples:
/// - Return a success with custom root fields: `JsonResponse::success_with(json!({ "user": user }))`
/// - Return a 400 error: `JsonResponse::error("Invalid input")`
/// - Return a 401 error: `JsonResponse::unauthorized("Unauthorized")`
/// - Log and return 500: `JsonResponse::fatal(err, "login query failed")`
pub struct JsonResponse;

impl JsonResponse {
    /// Convenience success that takes any serializable payload and nests it under `data`.
    pub fn success<T: Serialize>(data: T) -> HttpResponse {
        HttpResponse::Ok().json(json!({"success": "true","data": data}))
    }

    /// User-facing error helper based on ErrorBag.
    pub fn error(bag: ErrorBag) -> HttpResponse {
        HttpResponse::build(bag.status_code()).json(bag.to_json())
    }

    /// Internal Server Error (500) helper that logs the underlying error.
    ///
    /// - `err` is logged with `tracing::error` for observability.
    /// - Client gets a generic message (avoid leaking internals).
    pub fn fatal<E: std::fmt::Debug>(err: E, message: impl AsRef<str>) -> HttpResponse {
        tracing::error!(?err, "{}", message.as_ref());
        HttpResponse::InternalServerError().json(json!({
            "success": "false",
            "message": "Internal server error",
        }))
    }
}