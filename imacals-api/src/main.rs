mod config;
mod controllers;
mod routes;
mod services;
mod models;
mod repositories;
mod macros;
mod helpers;
mod utilities;
mod middlewares;
mod seeds;

use std::io;
use std::sync::{LazyLock};
use std::time::Duration;
use actix_cors::Cors;
use actix_extensible_rate_limit::backend::memory::InMemoryBackend;
use actix_extensible_rate_limit::backend::SimpleInputFunctionBuilder;
use actix_extensible_rate_limit::RateLimiter;
use actix_web::middleware::NormalizePath;
use actix_web::{web, App, HttpServer};
use reqwest_middleware::ClientWithMiddleware;
use sqlx::postgres::{PgPoolOptions};
use sqlx::{Pool, Postgres};
use tracing_actix_web::TracingLogger;

use crate::config::ENV;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::http_request::build_http_client;
use crate::utilities::tracing::{init_tracing, install_panic_hook, print_startup_banner};

pub static HTTP_CLIENT: LazyLock<ClientWithMiddleware> = LazyLock::new(|| build_http_client(Some(3)));

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    init_tracing();
    install_panic_hook();

    // Create PostgreSQL connection pool
    let pool = PgPoolOptions::new()
        .max_connections(64)
        .min_connections(4)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&ENV.database_url)
        .await
        .expect("Failed to connect to the database");

    // Migrations are embedded in the binary at compile time, so prod deploys don't need sqlx-cli.
    // NOTE: adding a migration is a SQL-only change; cargo won't rebuild on it, leaving these
    // embedded migrations stale (VersionMissing panic). Force a rebuild after adding one.
    sqlx::migrate!("./src/migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    // Seeds run in the background so a slow seed can't delay the port binding — while the socket
    // is closed anything proxying the API returns 502. Seeds are insert-only and idempotent, and
    // only add reference data, so serving a few requests while they finish is benign. Migrations
    // above stay blocking (the schema must exist before we serve). Order inside the task is
    // preserved for seeds that depend on each other.
    let seed_pool = pool.clone();
    actix_web::rt::spawn(async move {
        seeds::integration_seed::run(&seed_pool).await;
    });

    // Shared application state
    let state = AppState { pool: pool.clone() };


    // Custom JSON error handling to ensure
    // consistent error responses across the API
    let json_config = web::JsonConfig::default().error_handler(|err, _req| {
        ErrorBag::Deserialization(err.to_string()).into()
    });

    // Validator JSON configuration (for request validation)
    let validator_config = actix_web_validator::JsonConfig::default().error_handler(|err, _req| {
        ErrorBag::Deserialization(err.to_string()).into()
    });

    // One shared rate-limit store for the whole process — constructing it inside the
    // HttpServer::new closure would give every worker its own bucket (limit × workers).
    let rate_limit_backend = InMemoryBackend::builder().build();

    let server = HttpServer::new(move || {
        // Throttle 200 req/sec per IP
        let throttle = RateLimiter::builder(
            rate_limit_backend.clone(),
            SimpleInputFunctionBuilder::new(Duration::from_secs(1), 200).real_ip_key().build()
        ).add_headers().build();

        // API and Web CORS are separated
        // @todo: tightening
        let api_cors = Cors::permissive();
        let web_cors = Cors::permissive();

        App::new()
            .app_data(web::Data::new(state.clone()))
            .app_data(json_config.clone())
            .app_data(validator_config.clone())
            .service(web::scope("/api").wrap(api_cors).wrap(throttle).configure(routes::api::init))
            .service(web::scope("").wrap(web_cors).configure(routes::web::init))
            .wrap(TracingLogger::default())
            .wrap(NormalizePath::trim())
    })
        .bind(("0.0.0.0", ENV.app_port))?
        // Worker threads (usually = CPU cores)
        .workers(ENV.cpu_count)
        // Graceful shutdown timeout
        .shutdown_timeout(5);

    // Printed before .run() — awaiting the server first would hold the banner back until shutdown.
    print_startup_banner();

    server.run().await?;

    Ok(())
}
