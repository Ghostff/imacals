use crate::controllers::api::{
    api_auth_controller, default_controller, role_controller, user_controller,
};
use actix_web::web;
use actix_web::web::{delete, get, post, put};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", get().to(default_controller::health_check))
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
            web::scope("/users")
                .route("",      get().to(user_controller::index))
                .route("",      post().to(user_controller::create))
                .route("/{id}", get().to(user_controller::show))
                .route("/{id}", put().to(user_controller::update))
                .route("/{id}", delete().to(user_controller::delete))
        )
        .default_service(web::to(default_controller::page_not_found));
}
