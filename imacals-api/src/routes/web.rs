use actix_web::web;
use actix_web::web::get;
use crate::controllers::web::default_controller;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/", get().to(default_controller::index))
        .default_service(web::to(default_controller::page_not_found));
}