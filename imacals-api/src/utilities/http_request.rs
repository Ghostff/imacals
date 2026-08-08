use actix_web::{web, HttpRequest};
use std::collections::HashMap;

// Reads a query-string parameter off a request.
pub trait QueryRequestExt {
    fn get_query(&self, name: &str) -> Option<String>;
}

impl QueryRequestExt for HttpRequest {
    /// Get a value from the query string by name
    fn get_query(&self, name: &str) -> Option<String> {
        web::Query::<HashMap<String, String>>::from_query(self.query_string())
            .ok()
            .and_then(|query| query.get(name).cloned())
    }
}
