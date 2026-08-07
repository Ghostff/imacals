use actix_web::http::StatusCode;
use std::fmt;
use actix_web::HttpResponse;
use serde_json::{Map, Value};

/// ErrorBag: centralized, stable, AI-friendly application errors.
///
/// What it is:
/// - A small enum of known, client-facing error cases used across handlers/services.
/// - Each variant carries:
///   - status_code(): HTTP status to return
///   - error_code(): stable machine code (enum variant name)
///   - message(): short human-readable text
/// - Converts itself to a JSON body via to_json() and to an HttpResponse via Responder/ResponseError.
///
/// Why it exists:
/// - Single source of truth for user-facing errors (prevents ad-hoc strings/status codes).
/// - Stable automation surface: tools/clients/LLMs can reliably key off error_code.
/// - Safer API design: avoids leaking internals while keeping messages actionable.
///
/// AI usage guidelines:
/// - Prefer existing variants over inventing new ad-hoc strings.
/// - Choose the least-revealing variant for auth (e.g., InvalidEmailOrPassword).
/// - For field errors, use Validation { field, message } with concise, neutral text.
/// - For missing entities, use NotFound { entity } with a singular, lowercase entity name (e.g., "user").
/// - If you need a new business error:
///   1) Add a new enum variant
///   2) Map its StatusCode in status_code()
///   3) Provide a succinct message() string
///   4) Keep error_code() stable (do not rename variants casually)
///
/// JSON shape produced by to_json():
/// {
///   "error": { "message": "<human message>" },
///   "code":  "<stable_error_code>"
/// }
///
/// Interop:
/// - Handlers can return ErrorBag directly (Responder) or wrap via helpers that return HttpResponse.
/// - Downstream clients/agents should branch on "code", not on the "message".
#[derive(Debug, Clone)]
pub enum ErrorBag {
    /// Generic invalid credentials for login flows.
    /// Use this instead of revealing whether the email or password was incorrect.
    InvalidEmailOrPassword,
    Unauthorized,

    EmailInUse,
    InternalServerError(String),

    /// Entity-not-found style error with the entity name.
    NotFound(String),

    /// Validation error for a specific field with a custom message.
    Validation { field: String, message: String },

    /// Generic deserialization error
    Deserialization(String),

    Forbidden,
}

impl ErrorBag {
    /// Map each error to an appropriate HTTP status code.
    pub fn status_code(&self) -> StatusCode {
        match self {
            ErrorBag::InvalidEmailOrPassword => StatusCode::BAD_REQUEST,
            ErrorBag::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorBag::EmailInUse => StatusCode::CONFLICT,
            ErrorBag::NotFound(_) => StatusCode::NOT_FOUND,
            ErrorBag::Validation { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            ErrorBag::Deserialization(_) => StatusCode::BAD_REQUEST,
            ErrorBag::Forbidden => StatusCode::FORBIDDEN,
            ErrorBag::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Human-friendly message. Keep messages short and neutral.
    pub fn message(&self) -> String {
        match self {
            ErrorBag::InvalidEmailOrPassword => "Invalid email or password".to_string(),
            ErrorBag::Unauthorized => "Unauthorized".to_string(),
            ErrorBag::EmailInUse => "It looks like this email address is already registered.".to_string(),
            ErrorBag::NotFound(entity) => format!("{entity} not found"),
            ErrorBag::Validation { field, message } => format!("{field}: {message}"),
            ErrorBag::Deserialization(msg) => msg.clone(),
            ErrorBag::Forbidden => "You do not have permission to perform this action.".to_string(),
            ErrorBag::InternalServerError(msg) => msg.clone(),
        }
    }

    /// Stable error code equal to the enum variant name (e.g., "InvalidEmailOrPassword").
    pub fn error_code(&self) -> String {
        let dbg = format!("{:?}", self);
        let end = dbg.find(['(', ' ']).unwrap_or(dbg.len());

        dbg[..end].to_string()
    }

    pub fn to_json(&self) -> Value {
        let mut map = Map::new();
        let mut error_map = Map::new();
        error_map.insert("message".to_string(), Value::String(self.message()));

        map.insert("success".to_string(), Value::String("false".to_string()));
        map.insert("error".to_string(), Value::Object(error_map));
        map.insert("code".to_string(), Value::String(self.error_code()));

        map.into()
    }
}

impl fmt::Display for ErrorBag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl actix_web::ResponseError for ErrorBag {
    fn status_code(&self) -> StatusCode {
        // Call the inherent method via fully qualified syntax to avoid recursion
        ErrorBag::status_code(self)
    }

    fn error_response(&self) -> HttpResponse {
        // Also avoid the trait/inherent name clash on status_code.
        HttpResponse::build(ErrorBag::status_code(self)).json(self.to_json())
    }
}

impl actix_web::Responder for ErrorBag {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse {
        // Ensure we return the same concrete type (HttpResponse) as other branches
        HttpResponse::build(ErrorBag::status_code(&self)).json(self.to_json())
    }
}
