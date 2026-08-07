use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use serde_json::json;
use uuid::Uuid;

use crate::AppState;
use crate::models::organization::Organization;
use crate::models::user::User;
use crate::models::user_bank_account::CreateUserBankAccountSchema;
use crate::repositories::user_bank_account_repository::UserBankAccountRepository;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

pub async fn index(user: User, organization: Organization, app: Data<AppState>, id: Path<Uuid>) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "users.view");
    match UserBankAccountRepository::get_for_user(&app.pool, &id.into_inner()).await {
        Ok(accounts) => JsonResponse::success(accounts),
        Err(e)       => JsonResponse::fatal(e, "user_bank_account_controller.index failed"),
    }
}

pub async fn create(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
    body: Json<CreateUserBankAccountSchema>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "users.update");
    match UserBankAccountRepository::create(&app.pool, &id.into_inner(), &body).await {
        Ok(account) => JsonResponse::success(account),
        Err(e)      => JsonResponse::fatal(e, "user_bank_account_controller.create failed"),
    }
}

pub async fn update(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    path: Path<(Uuid, Uuid)>,
    body: Json<CreateUserBankAccountSchema>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "users.update");
    let (user_id, account_id) = path.into_inner();
    match UserBankAccountRepository::update(&app.pool, &account_id, &user_id, &body).await {
        Ok(0) => JsonResponse::error(ErrorBag::NotFound("Bank account".into())),
        Ok(_) => JsonResponse::success(json!({ "message": "Bank account updated successfully" })),
        Err(e) => JsonResponse::fatal(e, "user_bank_account_controller.update failed"),
    }
}

pub async fn delete(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    path: Path<(Uuid, Uuid)>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "users.update");
    let (user_id, account_id) = path.into_inner();
    match UserBankAccountRepository::delete(&app.pool, &account_id, &user_id).await {
        Ok(0) => JsonResponse::error(ErrorBag::NotFound("Bank account".into())),
        Ok(_) => JsonResponse::success(json!({ "message": "Bank account deleted successfully" })),
        Err(e) => JsonResponse::fatal(e, "user_bank_account_controller.delete failed"),
    }
}
