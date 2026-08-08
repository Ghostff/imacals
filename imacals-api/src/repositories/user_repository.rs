use crate::models::role::RoleSummary;
use crate::models::user::{User, UserWithRole};
use sqlx::{Error, PgPool};
use uuid::Uuid;

/// User repository encapsulates all SQL access for the `users` table.
///
/// Keeping queries in this layer makes controllers thin and focused on HTTP concerns.
pub struct UserRepository;

impl UserRepository {
    /// Find a user by their email where the account is not soft-deleted.
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<User, Error> {
        Ok(
            sqlx::query_as!(
                User,
                "SELECT * FROM users WHERE email = LOWER($1) AND deleted_at IS NULL LIMIT 1",
                email
            ).fetch_one(pool).await?
        )
    }

    /// Find a user by their id where the account is not soft-deleted.
    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<User, Error> {
        Ok(
            sqlx::query_as!(
                User,
                "SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL LIMIT 1",
                id
            ).fetch_one(pool).await?
        )
    }

    /// Updates an existing user record. Returns the number of affected rows.
    pub async fn update(pool: &PgPool, user: &User) -> Result<u64, Error> {
        Ok(
            sqlx::query!(
                r#"
                    UPDATE users
                    SET
                        first_name = $2,
                        last_name = $3,
                        email = LOWER($4),
                        phone = $5,
                        date_of_birth = $6,
                        password = $7,
                        password_reset_token = $8,
                        verification_token = $9,
                        last_logged_in_at = $10,
                        current_logged_in_at = $11,
                        role_id = $12,
                        updated_at = NOW(),
                        deleted_at = $13
                    WHERE id = $1
                    "#,
                user.id,
                user.first_name,
                user.last_name,
                user.email,
                user.phone,
                user.date_of_birth,
                user.password,
                user.password_reset_token,
                user.verification_token,
                user.last_logged_in_at,
                user.current_logged_in_at,
                user.role_id,
                user.deleted_at
            ).execute(pool).await?.rows_affected()
        )
    }

    /// Every active user with the role they hold.
    pub async fn index(pool: &PgPool) -> Result<Vec<UserWithRole>, Error> {
        let rows = sqlx::query!(
            r#"
            SELECT
                u.*,
                r.id    AS "r_id?: Uuid",
                r.name  AS "r_name?: String",
                r.title AS "r_title?: String"
            FROM users u
            LEFT JOIN roles r ON u.role_id = r.id AND r.deleted_at IS NULL
            WHERE u.deleted_at IS NULL
            ORDER BY u.first_name, u.last_name
            "#
        ).fetch_all(pool).await?;

        Ok(rows.into_iter().map(|row| {
            let role = match (row.r_id, row.r_name, row.r_title) {
                (Some(id), Some(name), Some(title)) => Some(RoleSummary { id, name, title }),
                _ => None,
            };
            UserWithRole {
                user: User {
                    id: row.id,
                    first_name: row.first_name,
                    last_name: row.last_name,
                    email: row.email,
                    phone: row.phone,
                    date_of_birth: row.date_of_birth,
                    password: row.password,
                    password_reset_token: row.password_reset_token,
                    is_superuser: row.is_superuser,
                    is_internal: row.is_internal,
                    verification_token: row.verification_token,
                    last_logged_in_at: row.last_logged_in_at,
                    current_logged_in_at: Some(row.current_logged_in_at),
                    role_id: row.role_id,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    deleted_at: row.deleted_at,
                },
                role,
            }
        }).collect())
    }

    /// True when the email is already taken by a live account.
    pub async fn email_exist(pool: &PgPool, needle: &str) -> Result<bool, Error> {
        Ok(sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = LOWER($1) AND deleted_at IS NULL)",
            needle
        ).fetch_one(pool).await?.exists.unwrap_or(false))
    }

    /// Creates a new user. `role_id` is optional so self-registration can create an account before
    /// anyone decides what the person does.
    pub async fn create(
        pool: &PgPool,
        f_name: &str,
        l_name: &str,
        email_address: &str,
        pass: &str,
        role_id: Option<&Uuid>,
    ) -> Result<User, Error> {
        Ok(
            sqlx::query_as!(
                User,
                "INSERT INTO users (first_name, last_name, email, password, role_id, current_logged_in_at)
                 VALUES ($1, $2, LOWER($3), $4, $5, NOW()) RETURNING *",
                f_name,
                l_name,
                email_address,
                pass,
                role_id,
            ).fetch_one(pool).await?
        )
    }

    /// Soft delete: keeps the row so past orders still resolve who handled them.
    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE users SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        ).execute(pool).await?.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(migrations = "./src/migrations")]
    async fn create_lowercases_the_email(pool: PgPool) {
        let user = UserRepository::create(&pool, "Chidi", "Okeke", "Chidi@Imacals.COM", "x", None)
            .await.unwrap();
        assert_eq!(user.email, "chidi@imacals.com");
    }

    // Lookup must be case-insensitive, or a customer who capitalises their email cannot log in.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn find_by_email_is_case_insensitive(pool: PgPool) {
        UserRepository::create(&pool, "Chidi", "Okeke", "chidi@imacals.com", "x", None).await.unwrap();
        assert!(UserRepository::find_by_email(&pool, "CHIDI@IMACALS.COM").await.is_ok());
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn soft_deleted_user_is_hidden(pool: PgPool) {
        let user = UserRepository::create(&pool, "Gone", "Away", "gone@imacals.com", "x", None)
            .await.unwrap();
        UserRepository::delete(&pool, &user.id).await.unwrap();

        assert!(UserRepository::find_by_id(&pool, &user.id).await.is_err());
        assert!(!UserRepository::email_exist(&pool, "gone@imacals.com").await.unwrap());
        assert!(!UserRepository::index(&pool).await.unwrap().iter().any(|u| u.user.id == user.id));
    }

    // Deleting twice must report zero rows the second time, so the controller can answer 404.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn deleting_twice_affects_no_rows(pool: PgPool) {
        let user = UserRepository::create(&pool, "A", "B", "twice@imacals.com", "x", None)
            .await.unwrap();

        assert_eq!(UserRepository::delete(&pool, &user.id).await.unwrap(), 1);
        assert_eq!(UserRepository::delete(&pool, &user.id).await.unwrap(), 0);
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn index_carries_the_users_role(pool: PgPool) {
        let role_id: Uuid = sqlx::query_scalar!("SELECT id FROM roles WHERE name = 'dispatch'")
            .fetch_one(&pool).await.unwrap();
        UserRepository::create(&pool, "Ada", "Nwosu", "ada@imacals.com", "x", Some(&role_id))
            .await.unwrap();

        let listed = UserRepository::index(&pool).await.unwrap();
        let ada = listed.iter().find(|u| u.user.email == "ada@imacals.com").unwrap();

        assert_eq!(ada.role.as_ref().map(|r| r.name.as_str()), Some("dispatch"));
    }

    // The seeded admin must come back with its role attached, not a null.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn seeded_admin_holds_the_admin_role(pool: PgPool) {
        let listed = UserRepository::index(&pool).await.unwrap();
        let admin = listed.iter().find(|u| u.user.email == "admin@imacals.com").unwrap();

        assert_eq!(admin.role.as_ref().map(|r| r.name.as_str()), Some("admin"));
        assert!(admin.user.is_superuser);
    }

    // The password hash must never reach a client.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn password_is_not_serialized(pool: PgPool) {
        let user = UserRepository::create(&pool, "A", "B", "secret@imacals.com", "hash", None)
            .await.unwrap();
        let json = serde_json::to_value(&user).unwrap();

        assert!(json.get("password").is_none());
        assert!(json.get("verification_token").is_none());
    }
}
