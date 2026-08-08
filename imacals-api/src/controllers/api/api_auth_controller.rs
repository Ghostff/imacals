use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web_validator::Json;
use chrono::Utc;
use serde_json::json;
use sqlx::Error;
use crate::AppState;
use crate::models::user::{LoginUserSchema, RegisterUserSchema, User};
use crate::repositories::user_repository::UserRepository;
use crate::services::crypto_service::CryptoService;
use crate::services::jwt_service::JwtService;
use crate::utilities::json_response::JsonResponse;
use crate::utilities::error_bag::ErrorBag;
use crate::services::user_service::UserService;

const JWT_TTL_MINUTES: i64 = 60 * 24;

pub async fn me(user: User) -> HttpResponse {
    JsonResponse::success(json!({ "user": user }))
}

/// Auth handlers: keep HTTP concerns here, push crypto/DB/JWT to services/repos.
/// Guarantees:
/// - Validated JSON via `actix-web-validator::Json<T>`.
/// - No user-enumeration: auth failures use a single generic message.
/// - Consistent envelope via `JsonResponse` (success/error/fatal).
pub async fn login(body: Json<LoginUserSchema>, app: Data<AppState>) -> HttpResponse {
    let email = body.email.trim();

    // Lookup user; hide existence via generic error.
    let mut user = match UserRepository::find_by_email(&app.pool, email).await {
        Ok(user) => user,
        Err(Error::RowNotFound) => return JsonResponse::error(ErrorBag::InvalidEmailOrPassword),
        Err(err) => return JsonResponse::fatal(err, "login.find_by_email failed"),
    };

    // Verify password; same generic error on mismatch.
    let crypto = CryptoService::new();
    if !crypto.verify_password(&user.password, &body.password) {
        return JsonResponse::error(ErrorBag::InvalidEmailOrPassword);
    }

    user.last_logged_in_at = user.current_logged_in_at;
    user.current_logged_in_at = Some(Utc::now());
    if let Err(e) = UserRepository::update(&app.pool, &user).await {
        return JsonResponse::fatal(e, "login.update failed");
    }

    // Issue 1-day JWT; prefix with Bearer for Authorization header use.
    let token = match JwtService::create_access_token(user.id, JWT_TTL_MINUTES) {
        Ok(t) => format!("Bearer {t}"),
        Err(e) => return JsonResponse::fatal(e, "login.create_access_token failed"),
    };

    JsonResponse::success(json!({ "user": user, "token": token }))
}

/// Public self-registration. The new account gets no role and therefore no permissions — an
/// administrator assigns one before the person can do anything in the dashboard.
pub async fn register(body: Json<RegisterUserSchema>, app: Data<AppState>) -> HttpResponse {
    let user = match UserService::create(
        &app.pool, &body.first_name, &body.last_name, &body.email, &body.password, None,
    ).await {
        Ok(u) => u,
        Err(e) => match e.status_code() {
            StatusCode::INTERNAL_SERVER_ERROR => return JsonResponse::fatal(e, ""),
            _ => return JsonResponse::error(e),
        },
    };

    let token = match JwtService::create_access_token(user.id, JWT_TTL_MINUTES) {
        Ok(t) => format!("Bearer {t}"),
        Err(e) => return JsonResponse::fatal(e, "register.create_access_token failed"),
    };

    JsonResponse::success(json!({ "user": user, "token": token }))
}
