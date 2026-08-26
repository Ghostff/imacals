use sqlx::{Error, PgPool};
use crate::models::user::{CreateUserSchema, User};
use crate::repositories::user_repository::UserRepository;
use crate::services::crypto_service::CryptoService;
use crate::utilities::error_bag::ErrorBag;
use crate::utilities::str::FilterEmptyString;

pub struct UserService;


impl UserService {
    pub async fn create(pool: &PgPool, body: &CreateUserSchema) -> Result<User, ErrorBag> {
        let password = body.password.as_deref().empty_as_none();
        let email = body.email.trim().to_lowercase();

        if password.is_none() {
            return Err(ErrorBag::Validation { field: "password".into(), message: "password is required".into() });
        }

        // Step 1: Early email existence check
        match UserRepository::email_exist(&pool, &email).await {
            Err(e) => return Err(ErrorBag::InternalServerError(format!("UserService::create.email_exist failed: {:?}", e))),
            Ok(true) => return Err(ErrorBag::EmailInUse),
            Ok(false) => {}
        };

        // Step 2: Hash password (Argon2)
        let crypto = CryptoService::new();
        let password = match crypto.hash_password(password.unwrap()) {
            Ok(hash) => hash,
            Err(e) => return Err(ErrorBag::InternalServerError(format!("UserService::create.hash_password failed: {:?}", e))),
        };

        // Step 3: Insert user
        match UserRepository::create(
            &pool,
            &body.first_name,
            &body.last_name,
            &email,
            body.phone.as_deref().empty_as_none(),
            &password,
        ).await {
            Ok(u) => Ok(u),
            Err(Error::Database(ref err)) if err.code().as_deref() == Some("23505") => {
                Err(ErrorBag::EmailInUse)
            },
            Err(e) => Err(ErrorBag::InternalServerError(format!("UserService::create.user.create failed: {:?}", e))),
        }
    }
}