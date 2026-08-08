use crate::repositories::user_repository::UserRepository;
use crate::services::jwt_service::JwtService;
use crate::utilities::error_bag::ErrorBag;
use crate::{AppState};
use actix_web::{dev::Payload, web, FromRequest, HttpRequest};
use futures::future::{LocalBoxFuture};
use uuid::Uuid;
use crate::models::user::User;
use crate::utilities::str::FilterEmptyString;

/// Attempts to resolve a `UserModel` from the incoming request by:
/// 1) Reading the `Authorization` header and extracting the Bearer token
/// 2) Verifying the token via `JwtService`
/// 3) Parsing the subject claim (`sub`) as a UUID
/// 4) Loading the user from the database
///
/// Returns `None` for any failure along the way.
async fn fetch_user_from_request(req: &HttpRequest) -> Option<User> {
    let token = match req
        .headers()
        .get("Authorization")
        .map(|h| h.to_str().unwrap())
        .empty_as_none()
        .unwrap_or_default()
        .strip_prefix("Bearer ")
    {
        Some(token) => token,
        None => return None,
    };

    let app = req.app_data::<web::Data<AppState>>()?.get_ref().clone();
    // Validate JWT and extract user ID
    let user_id = match JwtService::verify_access_token(token) {
        Ok(claims) => match Uuid::parse_str(&claims.sub) {
            Ok(id) => id,
            Err(_) => return None,
        },
        Err(_) => return None,
    };

    // Lookup user
    if let Ok(user) = UserRepository::find_by_id(&app.pool, &user_id).await {
        return Some(user);
    }

    None
}

/// Strict authentication extractor.
///
/// Fails with `ErrorBag::Unauthorized` when the request does not contain a valid Bearer token
/// and corresponding user record. Succeeds and yields the `UserModel` otherwise.
impl FromRequest for User {
    type Error = ErrorBag;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            match fetch_user_from_request(&req).await {
                Some(user) => Ok(user),
                None => Err(ErrorBag::Unauthorized),
            }
        })
    }
}
