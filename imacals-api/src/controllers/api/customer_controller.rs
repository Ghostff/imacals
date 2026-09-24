use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use serde_json::json;
use sqlx::Error;
use uuid::Uuid;

use crate::AppState;
use crate::models::customer::{CreateCustomerSchema, UpdateCustomerSchema};
use crate::models::organization::Organization;
use crate::models::user::User;
use crate::repositories::customer_repository::CustomerRepository;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

// List every active customer in the current organization.
pub async fn index(
    user: User,
    organization: Organization,
    app: Data<AppState>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "customers.view");
    match CustomerRepository::list_for_organization(&app.pool, &organization.id).await {
        Ok(c)  => JsonResponse::success(c),
        Err(e) => JsonResponse::fatal(e, "customer_controller.index failed"),
    }
}

// Look up a single customer by id.
pub async fn show(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "customers.view");
    match CustomerRepository::find_by_id(&app.pool, &id.into_inner()).await {
        Ok(c)                   => JsonResponse::success(c),
        Err(Error::RowNotFound) => JsonResponse::error(ErrorBag::NotFound("Customer".into())),
        Err(e)                  => JsonResponse::fatal(e, "customer_controller.show failed"),
    }
}

// Look up a customer by phone number — the staff path for taking a phone call.
pub async fn find_by_phone(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    phone: Path<String>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "customers.view");
    match CustomerRepository::find_by_phone(&app.pool, &organization.id, &phone.into_inner()).await {
        Ok(Some(c)) => JsonResponse::success(c),
        Ok(None)    => JsonResponse::error(ErrorBag::NotFound("Customer".into())),
        Err(e)      => JsonResponse::fatal(e, "customer_controller.find_by_phone failed"),
    }
}

// Create a customer. Used when staff take a phone call from someone who isn't on file yet.
pub async fn create(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    body: Json<CreateCustomerSchema>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "customers.create");
    match CustomerRepository::create(
        &app.pool,
        &organization.id,
        &body.full_name,
        body.phone.as_deref(),
        body.email.as_deref(),
        body.user_id.as_ref(),
        &user.id,
    )
    .await
    {
        Ok(c) => JsonResponse::success(c),
        Err(e) => JsonResponse::fatal(e, "customer_controller.create failed"),
    }
}

// Update a customer.
pub async fn update(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
    body: Json<UpdateCustomerSchema>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "customers.update");
    let customer_id = id.into_inner();

    let mut customer = match CustomerRepository::find_by_id(&app.pool, &customer_id).await {
        Ok(c) => c,
        Err(Error::RowNotFound) => return JsonResponse::error(ErrorBag::NotFound("Customer".into())),
        Err(e) => return JsonResponse::fatal(e, "customer_controller.update failed to load"),
    };

    if customer.organization_id != organization.id {
        return JsonResponse::error(ErrorBag::NotFound("Customer".into()));
    }

    if let Some(ref name) = body.full_name {
        customer.full_name = name.clone();
    }
    if body.phone.is_some() {
        customer.phone = body.phone.clone();
    }
    if body.email.is_some() {
        customer.email = body.email.clone();
    }

    match CustomerRepository::update(&app.pool, &customer).await {
        Ok(c) => JsonResponse::success(c),
        Err(e) => JsonResponse::fatal(e, "customer_controller.update failed"),
    }
}

// Soft-delete a customer. The cascade trigger soft-deletes their wishlists.
pub async fn delete(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "customers.delete");
    match CustomerRepository::delete(&app.pool, &id.into_inner()).await {
        Ok(0)  => JsonResponse::error(ErrorBag::NotFound("Customer".into())),
        Ok(_)  => JsonResponse::success(json!({ "message": "Customer deleted successfully" })),
        Err(e) => JsonResponse::fatal(e, "customer_controller.delete failed"),
    }
}
