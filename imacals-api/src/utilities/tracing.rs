use std::{env, io, panic};
use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::config::ENV;

/// Initialize tracing:
/// - Default: JSON to stdout (best for CloudWatch).
/// - When LOG_FORMAT=pretty -> human-friendly console output.
/// - Level comes from RUST_LOG (e.g. "info,mycrate=debug").
pub fn init_tracing() {
    // Default filter if RUST_LOG not set.
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,actix_web=info,sqlx=warn"));
    let format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "json".into());

    // ---------- PRETTY (LOCAL DEV) ----------
    if format == "pretty" {
        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .with_writer(io::stdout)
                    .with_ansi(true)
                    .compact()                     // cleaner than default
                    .with_target(false)            // remove noisy module path
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .with_span_events(FmtSpan::CLOSE)
                    .with_timer(fmt::time::UtcTime::rfc_3339()),
            )
            .init();

        return;
    }

    // ---------- JSON (PRODUCTION) ----------
    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_writer(io::stdout)
                .json()
                .with_current_span(false)      // remove span duplication
                .with_span_list(false)         // reduces nesting noise
                .with_target(true)
                .with_file(false)              // remove unless debugging prod
                .with_line_number(false)
                .with_span_events(FmtSpan::CLOSE)
                .with_timer(fmt::time::UtcTime::rfc_3339()),
        )
        .init();
}

pub fn install_panic_hook() {
    // Capture panics and log them through `tracing` so they show in CloudWatch.
    panic::set_hook(Box::new(|info| {
        // Extract panic message
        let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
            *s
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.as_str()
        } else {
            "panic payload not a string"
        };

        let (file, line, col) = info.location()
            .map(|l| (l.file(), l.line(), l.column()))
            .unwrap_or(("<unknown>", 0, 0));

        // Capture a backtrace and filter to only relevant (project) frames
        let raw = std::backtrace::Backtrace::force_capture().to_string();
        let backtrace: String = raw
            .lines()
            .filter(|l| l.contains("rental_core") || l.contains("src/"))
            .collect::<Vec<_>>()
            .join("\n");

        tracing::error!(
            target: "panic",
            message = %msg,
            file = %file,
            line = line,
            column = col,
            backtrace = %backtrace,
            "application panic"
        );
    }));
}

pub fn print_startup_banner() {
    // Local banner: the browsable URLs first (the front ends are what a developer opens), then the
    // API, then internal state. URLs only — never echo credentials into logs. Every port is read
    // from the variable the service is actually configured with, so the banner can't advertise an
    // address nothing is listening on.
    if ENV.app_env == "local" {
        let dashboard_port = env::var("DASHBOARD_HOST_PORT").unwrap_or_else(|_| "5174".to_string());
        let web_port       = env::var("WEB_HOST_PORT").unwrap_or_else(|_| "5175".to_string());

        eprintln!(
            "\n┌─────────────────────────────────────────┐\n\
               │           🚀  Server Started             │\n\
               └─────────────────────────────────────────┘\n\
               \n  🛒  Storefront → http://localhost:{}\
               \n  🖥️  Dashboard  → http://localhost:{}\
               \n  🌐  API        → {}:{}\
               \n  🗄️  Database   → connected\
               \n  ⚙️  Workers    → {}\
               \n  🔧  Env        → {}\n",
            web_port,
            dashboard_port,
            ENV.app_url, ENV.app_port,
            ENV.cpu_count,
            ENV.app_env,
        );

        return;
    }

    eprintln!(
        "\n┌─────────────────────────────────────────┐\n\
           │           🚀  Server Started             │\n\
           └─────────────────────────────────────────┘\n\
           \n  🌐  API        → {}:{}\
           \n  🗄️  Database   → connected\
           \n  ⚙️  Workers    → {}\
           \n  🔧  Env        → {}\n",
        ENV.app_url, ENV.app_port,
        ENV.cpu_count,
        ENV.app_env,
    );
}