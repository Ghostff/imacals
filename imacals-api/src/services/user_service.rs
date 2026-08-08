use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::models::user::User;
use crate::repositories::permission_repository::PermissionRepository;
use crate::repositories::user_repository::UserRepository;
use crate::services::crypto_service::CryptoService;
use crate::utilities::error_bag::ErrorBag;

pub struct UserService;

impl UserService {
    /// Creates a user and, when a role is given, grants that role's permission bundle.
    ///
    /// `role_id` is None for public self-registration: an unauthenticated caller must never be able
    /// to pick its own permissions.
    pub async fn create(
        pool: &PgPool,
        first_name: &str,
        last_name: &str,
        email: &str,
        password: &str,
        role_id: Option<&Uuid>,
    ) -> Result<User, ErrorBag> {
        let email = email.trim().to_lowercase();

        if password.trim().is_empty() {
            return Err(ErrorBag::Validation { field: "password".into(), message: "password is required".into() });
        }

        match UserRepository::email_exist(pool, &email).await {
            Err(e) => return Err(ErrorBag::InternalServerError(format!("UserService::create.email_exist failed: {:?}", e))),
            Ok(true) => return Err(ErrorBag::EmailInUse),
            Ok(false) => {}
        };

        let crypto = CryptoService::new();
        let hashed = match crypto.hash_password(password) {
            Ok(hash) => hash,
            Err(e) => return Err(ErrorBag::InternalServerError(format!("UserService::create.hash_password failed: {:?}", e))),
        };

        let user = match UserRepository::create(pool, first_name, last_name, &email, &hashed, role_id).await {
            Ok(u) => u,
            // The pre-check above races: two concurrent signups can both see a free email.
            // The unique index is the real guard, so map its violation to the same error.
            Err(Error::Database(ref err)) if err.code().as_deref() == Some("23505") => {
                return Err(ErrorBag::EmailInUse)
            },
            Err(e) => return Err(ErrorBag::InternalServerError(format!("UserService::create.user.create failed: {:?}", e))),
        };

        if let Some(role_id) = role_id {
            if let Err(e) = PermissionRepository::sync_from_role(pool, &user.id, role_id).await {
                return Err(ErrorBag::InternalServerError(format!("UserService::create.sync_from_role failed: {:?}", e)));
            }
        }

        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(migrations = "./src/migrations")]
    async fn create_rejects_a_blank_password(pool: PgPool) {
        let result = UserService::create(&pool, "A", "B", "blank@imacals.com", "   ", None).await;
        assert!(matches!(result, Err(ErrorBag::Validation { .. })));
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn create_rejects_a_duplicate_email(pool: PgPool) {
        UserService::create(&pool, "A", "B", "dupe@imacals.com", "secret123", None).await.unwrap();
        let second = UserService::create(&pool, "C", "D", "DUPE@imacals.com", "secret123", None).await;

        assert!(matches!(second, Err(ErrorBag::EmailInUse)));
    }

    // The stored password must be an Argon2 hash, never the plaintext.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn create_hashes_the_password(pool: PgPool) {
        let user = UserService::create(&pool, "A", "B", "hash@imacals.com", "secret123", None)
            .await.unwrap();

        assert_ne!(user.password, "secret123");
        assert!(user.password.starts_with("$argon2"));
        assert!(CryptoService::new().verify_password(&user.password, "secret123"));
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn create_with_a_role_grants_that_roles_permissions(pool: PgPool) {
        let role_id: Uuid = sqlx::query_scalar!("SELECT id FROM roles WHERE name = 'order-desk'")
            .fetch_one(&pool).await.unwrap();

        let user = UserService::create(&pool, "Ada", "N", "desk@imacals.com", "secret123", Some(&role_id))
            .await.unwrap();

        assert_eq!(user.role_id, Some(role_id));
        assert!(PermissionRepository::can(&pool, &user.id, "users.view").await.unwrap());
        assert!(!PermissionRepository::can(&pool, &user.id, "users.delete").await.unwrap());
    }

    // Self-registration must leave the account with no permissions at all.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn create_without_a_role_grants_nothing(pool: PgPool) {
        let user = UserService::create(&pool, "A", "B", "norole@imacals.com", "secret123", None)
            .await.unwrap();

        assert!(user.role_id.is_none());
        assert!(!PermissionRepository::can(&pool, &user.id, "users.view").await.unwrap());
        assert!(!user.is_superuser);
    }
}
