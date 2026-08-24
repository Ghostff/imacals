use crate::controllers::api::{
    api_auth_controller, default_controller, domain_controller, domain_system_user_controller,
    geo_controller, organization_controller, organization_user_role_controller, polygon_controller,
    polygon_neighbor_controller, polygon_zone_controller, role_controller, user_controller,
    user_document_controller, user_bank_account_controller,
    integration_controller, attribute_controller,
    catalog_controller, product_controller, category_controller,
};
use actix_web::web;
use actix_web::web::{delete, get, post, put};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", get().to(default_controller::health_check))
        .service(
            web::scope("/catalog")
                .route("/products", get().to(catalog_controller::products))
                .route("/products/{slug}", get().to(catalog_controller::show_by_slug))
                .route("/categories", get().to(catalog_controller::categories))
        )
        .service(
            web::scope("/products")
                .route("", get().to(product_controller::index))
                .route("", post().to(product_controller::create))
                .route("/{id}", get().to(product_controller::show))
                .route("/{id}", put().to(product_controller::update))
                .route("/{id}", delete().to(product_controller::delete))
                .route("/{id}/image", post().to(product_controller::upload_image))
        )
        .service(
            web::scope("/categories")
                .route("", get().to(category_controller::index))
                .route("", post().to(category_controller::create))
                .route("/{id}", get().to(category_controller::show))
                .route("/{id}", put().to(category_controller::update))
                .route("/{id}", delete().to(category_controller::delete))
        )
        .service(
            web::scope("/auth")
                .route("/me", get().to(api_auth_controller::me))
                .route("/login", post().to(api_auth_controller::login))
                .route("/register", post().to(api_auth_controller::register))
        )
        .service(
            web::scope("/roles")
                .route("", get().to(role_controller::index))
        )
        .service(
            web::scope("/user-roles")
                .route("", get().to(organization_user_role_controller::index))
        )
        .service(
            web::scope("/users")
                .route("", get().to(user_controller::index))
                .route("", post().to(user_controller::create))
                .route("/{id}", get().to(user_controller::show))
                .route("/{id}", put().to(user_controller::update))
                .route("/{id}", delete().to(user_controller::delete))
                // Documents (signature, initials, proof_of_funds)
                .route("/{id}/documents", get().to(user_document_controller::index))
                .route("/{id}/documents", post().to(user_document_controller::create))
                .route("/{id}/documents/{doc_id}", delete().to(user_document_controller::delete))
                // Bank accounts
                .route("/{id}/bank-accounts", get().to(user_bank_account_controller::index))
                .route("/{id}/bank-accounts", post().to(user_bank_account_controller::create))
                .route("/{id}/bank-accounts/{account_id}", put().to(user_bank_account_controller::update))
                .route("/{id}/bank-accounts/{account_id}", delete().to(user_bank_account_controller::delete))
        )
        .service(
            web::scope("/organizations")
                .route("", get().to(organization_controller::index))
                .route("", post().to(organization_controller::create))
                .route("/{id}", get().to(organization_controller::show))
                .route("/{id}", put().to(organization_controller::update))
                .route("/{id}", delete().to(organization_controller::delete))
        )
        .service(
            web::scope("/geo")
                .route("/countries",                              get().to(geo_controller::countries))
                .route("/countries/{country_id}/states",          get().to(geo_controller::states))
                .route("/states/{state_id}/cities",               get().to(geo_controller::cities))
        )
        .service(
            web::scope("/polygons")
                .route("",          get().to(polygon_controller::index))
                .route("",          post().to(polygon_controller::create))
                .route("/{id}",     get().to(polygon_controller::show))
                .route("/{id}",     put().to(polygon_controller::update))
                .route("/{id}",     delete().to(polygon_controller::delete))
                .route("/{id}/polygon-zone", put().to(polygon_controller::assign_polygon_zone))
        )
        .service(
            web::scope("/polygon-neighbors")
                .route("",                              get().to(polygon_neighbor_controller::index))
                .route("",                              post().to(polygon_neighbor_controller::create))
                .route("/{polygon_id}/{neighbor_id}",   delete().to(polygon_neighbor_controller::delete))
        )
        .service(
            web::scope("/polygon-zones")
                .route("",      get().to(polygon_zone_controller::index))
                .route("",      post().to(polygon_zone_controller::create))
                .route("/{id}", put().to(polygon_zone_controller::update))
        )
        .service(
            web::scope("/domains")
                .route("",      get().to(domain_controller::index))
                .route("",      post().to(domain_controller::create))
                .route("/{id}", get().to(domain_controller::show))
                .route("/{id}", put().to(domain_controller::update))
                .route("/{id}", delete().to(domain_controller::delete))
        )
        .service(
            web::scope("/integrations")
                .route("",                      get().to(integration_controller::index))
                .route("",                      post().to(integration_controller::create))
                // Before /{id} so "provider-types" isn't parsed as a UUID path segment.
                .route("/provider-types",       get().to(integration_controller::provider_types))
                .route("/{id}",                 get().to(integration_controller::show))
                .route("/{id}",                 put().to(integration_controller::update))
                .route("/{id}",                 delete().to(integration_controller::delete))
                .route("/{id}/enabled",         put().to(integration_controller::set_enabled))
                .route("/{id}/attributes",      get().to(integration_controller::attributes))
        )
        .service(
            web::scope("/attributes")
                .route("",      post().to(attribute_controller::create))
                .route("/{id}", put().to(attribute_controller::update))
                .route("/{id}", delete().to(attribute_controller::delete))
        )
        .service(
            web::scope("/domain-system-users")
                .route("",                get().to(domain_system_user_controller::index))
                .route("/eligible-roles", get().to(domain_system_user_controller::eligible_roles))
                .route("",                post().to(domain_system_user_controller::upsert))
                .route("/{id}",           delete().to(domain_system_user_controller::delete))
        )
        .default_service(web::to(default_controller::page_not_found));
}
