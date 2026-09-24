use actix_web::web::{Data, Json, Path};
use actix_web::HttpResponse;
use serde_json::json;
use uuid::Uuid;

use crate::AppState;
use crate::models::organization::Organization;
use crate::models::user::User;
use crate::models::wishlist::{
    AddWishlistItemSchema, CreateWishlistSchema, UpdateWishlistSchema,
};
use crate::repositories::wishlist_repository::WishlistRepository;
use crate::services::wishlist_service::WishlistService;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::json_response::JsonResponse;

// List the calling customer's wishlists. Service ensures the customer is resolved from the
// authenticated user, so a storefront customer only ever sees their own.
pub async fn index(
    user: User,
    organization: Organization,
    app: Data<AppState>,
) -> HttpResponse {
    match WishlistService::list_for_customer(&app.pool, &organization.id, &user.id).await {
        Ok(items) => JsonResponse::success(items),
        Err(err_bag) => JsonResponse::error(err_bag),
    }
}

// Show a single wishlist with its items.
pub async fn show(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
) -> HttpResponse {
    match WishlistService::show(&app.pool, &organization.id, &user.id, &id.into_inner()).await {
        Ok(detail)   => JsonResponse::success(detail),
        Err(err_bag) => JsonResponse::error(err_bag),
    }
}

// Create a new wishlist for the calling customer.
pub async fn create(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    body: Json<CreateWishlistSchema>,
) -> HttpResponse {
    match WishlistService::create(&app.pool, &organization.id, &user.id, &body).await {
        Ok(w)        => JsonResponse::success(w),
        Err(err_bag) => JsonResponse::error(err_bag),
    }
}

// Update a wishlist the caller owns.
pub async fn update(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
    body: Json<UpdateWishlistSchema>,
) -> HttpResponse {
    match WishlistService::update(&app.pool, &organization.id, &user.id, &id.into_inner(), &body)
        .await
    {
        Ok(w)        => JsonResponse::success(w),
        Err(err_bag) => JsonResponse::error(err_bag),
    }
}

// Soft-delete a wishlist the caller owns. Cascade trigger removes its items.
pub async fn delete(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
) -> HttpResponse {
    match WishlistService::delete(&app.pool, &organization.id, &user.id, &id.into_inner()).await {
        Ok(()) => JsonResponse::success(json!({ "message": "Wishlist deleted successfully" })),
        Err(err_bag) => JsonResponse::error(err_bag),
    }
}

// Add a product to a wishlist.
pub async fn add_item(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    id: Path<Uuid>,
    body: Json<AddWishlistItemSchema>,
) -> HttpResponse {
    match WishlistService::add_item(
        &app.pool,
        &organization.id,
        &user.id,
        &id.into_inner(),
        &body,
    )
    .await
    {
        Ok(detail)   => JsonResponse::success(detail),
        Err(err_bag) => JsonResponse::error(err_bag),
    }
}

// Remove an item from a wishlist.
pub async fn remove_item(
    user: User,
    organization: Organization,
    app: Data<AppState>,
    path: Path<(Uuid, Uuid)>,
) -> HttpResponse {
    let (wishlist_id, item_id) = path.into_inner();
    match WishlistService::remove_item(&app.pool, &organization.id, &user.id, &wishlist_id, &item_id)
        .await
    {
        Ok(())       => JsonResponse::success(json!({ "message": "Item removed from wishlist" })),
        Err(err_bag) => JsonResponse::error(err_bag),
    }
}

// Staff overview: list every wishlist in the active organization.
pub async fn admin_index(
    user: User,
    organization: Organization,
    app: Data<AppState>,
) -> HttpResponse {
    crate::gate!(&app.pool, &user, &organization, "wishlists.view");
    match WishlistRepository::list_for_organization(&app.pool, &organization.id).await {
        Ok(items) => JsonResponse::success(items),
        Err(e)    => JsonResponse::fatal(e, "wishlist_controller.admin_index failed"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use serde_json::json;
    use sqlx::PgPool;

    use crate::services::jwt_service::JwtService;

    // Insert a user and return their Bearer token + the org id they belong to via X-Org header.
    async fn make_user_in_org(pool: &PgPool, email: &str) -> (Uuid, Uuid, String) {
        let org_id = sqlx::query_scalar!(
            "INSERT INTO organizations (name, slug) VALUES ('Imacals', $1) RETURNING id",
            format!("org-{}", email)
        )
        .fetch_one(pool)
        .await
        .unwrap();
        let user_id = sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, current_logged_in_at)
             VALUES ('T','T',$1,'x',NOW()) RETURNING id",
            email
        )
        .fetch_one(pool)
        .await
        .unwrap();
        let token = format!("Bearer {}", JwtService::create_access_token(user_id, 60).unwrap());
        (user_id, org_id, token)
    }

    // Seed a customer row linked to the user so the service can resolve it.
    async fn link_customer(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> Uuid {
        crate::repositories::customer_repository::CustomerRepository::create(
            pool, &org_id, "Tester", Some("08030000000"), None, Some(&user_id), &user_id,
        )
        .await
        .unwrap()
        .id
    }

    // A request without a Bearer token must be rejected before the handler runs.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn list_without_token_returns_401(pool: PgPool) {
        let (_uid, org_id, _t) = make_user_in_org(&pool, "anon@test.com").await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/wishlists").route("", web::get().to(index))),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/wishlists")
            .insert_header(("X-Organization-Id", org_id.to_string()))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // One customer's wishlists must never leak to another customer.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn customer_cannot_see_another_customers_wishlists(pool: PgPool) {
        let (_alice, org_id, alice_token) = make_user_in_org(&pool, "alice@test.com").await;
        let (_bob, _, bob_token) = make_user_in_org(&pool, "bob@test.com").await;

        // Alice has a customer + wishlist; Bob has a customer but no wishlist.
        let alice_uid = sqlx::query_scalar!(
            "SELECT id FROM users WHERE email = 'alice@test.com'"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let bob_uid = sqlx::query_scalar!(
            "SELECT id FROM users WHERE email = 'bob@test.com'"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        link_customer(&pool, org_id, alice_uid).await;
        link_customer(&pool, org_id, bob_uid).await;

        // Alice creates a wishlist.
        let app_alice = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/wishlists").route("", web::post().to(create))),
        )
        .await;
        let create_req = test::TestRequest::post()
            .uri("/wishlists")
            .insert_header(("Authorization", alice_token.clone()))
            .insert_header(("X-Organization-Id", org_id.to_string()))
            .set_json(json!({ "name": "Alice's list" }))
            .to_request();
        assert_eq!(test::call_service(&app_alice, create_req).await.status(), StatusCode::OK);

        // Bob lists — must not see Alice's list.
        let app_bob = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/wishlists").route("", web::get().to(index))),
        )
        .await;
        let list_req = test::TestRequest::get()
            .uri("/wishlists")
            .insert_header(("Authorization", bob_token))
            .insert_header(("X-Organization-Id", org_id.to_string()))
            .to_request();
        let resp = test::call_service(&app_bob, list_req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = test::read_body_json(resp).await;
        let arr = body["data"].as_array().expect("data must be an array");
        assert!(arr.is_empty(), "Bob must not see Alice's wishlist");
    }

    // Showing someone else's wishlist must 404 — never 403, so the existence isn't leaked.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn show_others_wishlist_returns_404(pool: PgPool) {
        let (_alice, org_id, alice_token) = make_user_in_org(&pool, "a2@test.com").await;
        let (_bob, _, bob_token) = make_user_in_org(&pool, "b2@test.com").await;
        let alice_uid = sqlx::query_scalar!(
            "SELECT id FROM users WHERE email = 'a2@test.com'"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        link_customer(&pool, org_id, alice_uid).await;

        // Alice creates.
        let alice_app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/wishlists").route("", web::post().to(create))),
        )
        .await;
        let resp = test::call_service(
            &alice_app,
            test::TestRequest::post()
                .uri("/wishlists")
                .insert_header(("Authorization", alice_token.clone()))
                .insert_header(("X-Organization-Id", org_id.to_string()))
                .set_json(json!({ "name": "Private" }))
                .to_request(),
        )
        .await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        let wishlist_id = body["data"]["id"].as_str().unwrap().to_string();

        // Bob tries to fetch it.
        let bob_app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState { pool: pool.clone() }))
                .service(web::scope("/wishlists").route("/{id}", web::get().to(show))),
        )
        .await;
        let resp = test::call_service(
            &bob_app,
            test::TestRequest::get()
                .uri(&format!("/wishlists/{}", wishlist_id))
                .insert_header(("Authorization", bob_token))
                .insert_header(("X-Organization-Id", org_id.to_string()))
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
