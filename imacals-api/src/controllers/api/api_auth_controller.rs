use actix_web::{HttpResponse};
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web_validator::Json;
use chrono::Utc;
use serde_json::{json, to_value, Map, Value};
use sqlx::Error;
use crate::AppState;
use crate::models::user::{LoginUserSchema, CreateUserSchema, User};
use crate::repositories::user_repository::UserRepository;
use crate::services::crypto_service::CryptoService;
use crate::services::jwt_service::JwtService;
use crate::utilities::json_response::JsonResponse;
use crate::utilities::error_bag::ErrorBag;
use crate::repositories::organization_repository::OrganizationRepository;
use crate::repositories::organization_user_repository::OrganizationUserRepository;
use crate::services::user_service::UserService;

const JWT_TTL_MINUTES: i64 = 60 * 24;

async fn build_user_response(user: User, app: &AppState) -> Map<String, Value> {
    let mut map = Map::new();

    let organizations = OrganizationRepository::get_all_with_permissions_for_user(&app.pool, &user)
        .await
        .unwrap_or_default();

    map.insert("user".to_string(), to_value(user).unwrap_or_default());
    map.insert("organizations".to_string(), to_value(organizations).unwrap_or_default());
    map
}


pub async fn me(user: User, app: Data<AppState>) -> HttpResponse {
    JsonResponse::success(json!(build_user_response(user, &app).await))
}

/// Auth handlers: keep HTTP concerns here, push crypto/DB/JWT to services/repos.
/// Guarantees:
/// - Validated JSON via `actix-web-validator::Json<T>`.
/// - No user-enumeration: auth failures use a single generic message.
/// - Consistent envelope via `JsonResponse` (success/error/fatal).
///
/// Security:
/// - Prevents user enumeration
/// - Avoids timing side-channel leaks
pub async fn login(body: Json<LoginUserSchema>, app: Data<AppState>) -> HttpResponse {
    // Input is validated already; normalize email.
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

    // Update login timestamps; ignore details in client response on failure.
    user.last_logged_in_at = user.current_logged_in_at.clone();
    user.current_logged_in_at = Some(Utc::now());
    if let Err(e) = UserRepository::update(&app.pool, &user).await {
        return JsonResponse::fatal(e, "login.update failed");
    }

    // Issue 1-day JWT; prefix with Bearer for Authorization header use.
    let token = match JwtService::create_access_token(user.id, JWT_TTL_MINUTES) {
        Ok(t) => format!("Bearer {t}"),
        Err(e) => return JsonResponse::fatal(e, "register.create_access_token failed"),
    };
    
    let mut response = build_user_response(user, &app).await;
    response.insert("token".to_string(), to_value(token).unwrap_or_default());

    JsonResponse::success(json!(response))
}

/// Registration flow:
/// - Ensures unique email (case-insensitive, pre-insert check).
/// - Hashes password securely using Argon2.
/// - Wraps insert and any hooks in a transaction (atomic).
/// - Handles duplicate key errors (23505) gracefully.
/// - Returns 201 Created + { user, token } on success.
///
/// Security:
/// - No password hash exposure.
/// - Email is normalized (trim + lowercase) before any DB call.
pub async fn register(body: Json<CreateUserSchema>, app: Data<AppState>) -> HttpResponse {
    let user = match UserService::create(&app.pool, &body).await {
        Ok(u) => u,
        Err(e) => match e.status_code() {
            StatusCode::INTERNAL_SERVER_ERROR => return JsonResponse::fatal(e, ""),
            _ => return JsonResponse::error(e),
        },
    };

    // Add new user to the default "imacals" organization
    let imacals_org = match OrganizationRepository::find_by_slug(&app.pool, "imacals").await {
        Ok(o) => o,
        Err(e) => return JsonResponse::fatal(e, "register.find_imacals_org failed"),
    };
    if let Err(e) = OrganizationUserRepository::sync_user_organizations_and_permissions(
        &app.pool, &user, &[imacals_org.id], &user,
    ).await {
        return JsonResponse::fatal(e, "register.sync_org failed");
    }

    let token = match JwtService::create_access_token(user.id, JWT_TTL_MINUTES) {
        Ok(t) => format!("Bearer {t}"),
        Err(e) => return JsonResponse::fatal(e, "register.create_access_token failed"),
    };

    JsonResponse::success(json!({"user": user,"token": token}))
}
