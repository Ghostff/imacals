use actix_web::{web, HttpRequest};
use std::collections::HashMap;
use std::time::Duration;
use reqwest::Client;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::policies::ExponentialBackoff;
use reqwest_retry::RetryTransientMiddleware;
use reqwest_tracing::TracingMiddleware;

pub trait OrganizationRequestExt {
    fn get_query(&self, name: &str) -> Option<String>;
    fn get_header(&self, name: &str) -> Option<String>;
    fn get_query_or_header(&self, query_name: &str, header_name: &str) -> Option<String>;
}

impl OrganizationRequestExt for HttpRequest {
    /// Get a value from the query string by name
    fn get_query(&self, name: &str) -> Option<String> {
        web::Query::<HashMap<String, String>>::from_query(self.query_string())
            .ok()
            .and_then(|query| query.get(name).cloned())
    }

    /// Get a value from an HTTP header by name
    fn get_header(&self, name: &str) -> Option<String> {
        self.headers()
            .get(name)
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
    }

    /// Get a value from query string first, then fall back to header
    fn get_query_or_header(&self, query_name: &str, header_name: &str) -> Option<String> {
        self.get_query(query_name).or_else(|| self.get_header(header_name))
    }
}

pub fn build_http_client(retries: Option<u32>) -> ClientWithMiddleware {
    let base_client = Client::builder()
        .timeout(Duration::from_secs(30))
        .danger_accept_invalid_certs(true)
        .default_headers({
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert(
                reqwest::header::USER_AGENT,
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.212 Safari/537.36".parse().unwrap(),
            );
            headers
        })
        .build()
        .expect("Could not build base HTTP client");

    let mut client = ClientBuilder::new(base_client).with(TracingMiddleware::default());

    if let Some(retries) = retries {
        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(Duration::from_millis(100), Duration::from_secs(2))
            .build_with_max_retries(retries);

        client = client.with(RetryTransientMiddleware::new_with_policy(retry_policy));
    }

    client.build()
}

