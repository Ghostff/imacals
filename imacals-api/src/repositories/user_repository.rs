use crate::models::organization_user_role::OrganizationUserRoleSummary;
use crate::models::role::RoleSummary;
use crate::models::user::{User, UserWithOrganizations};
use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::models::organization::Organization;

/// User repository encapsulates all SQL access for the `users` table.
///
/// Keeping queries in this layer makes controllers thin and focused on HTTP concerns.
pub struct UserRepository;

impl UserRepository {
    /// Find a user by their email where the account is not soft-deleted.
    ///
    /// Uses SQLx compile-time checked query macros ("sqlx micro") for safety.
    pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<User, Error> {
        // Select all columns so we can hydrate the full domain model.
        Ok(
            sqlx::query_as!(
                User,
                "SELECT * FROM users WHERE email = LOWER($1) AND deleted_at IS NULL LIMIT 1",
                email
            ).fetch_one(pool).await?
        )
    }

    /// Find a user by their id where the account is not soft-deleted.
    ///
    /// Uses SQLx compile-time checked query macros ("sqlx micro") for safety.
    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<User, Error> {
        // Select all columns so we can hydrate the full domain model.
        Ok(
            sqlx::query_as!(
                User,
                "SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL LIMIT 1",
                id
            ).fetch_one(pool).await?
        )
    }

    /// Updates an existing user record in the database.
    ///
    /// Takes a user model and updates all fields in the database. Returns the number of affected rows.
    /// Uses SQLx compile-time checked query for type safety.
    pub async fn update(pool: &PgPool, user: &User) -> Result<u64, Error> {
        // Use SQLx compile-time checked query and return affected rows for AI-friendly handling.
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
                        updated_at = NOW(),
                        deleted_at = $12
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
                user.deleted_at
            ).execute(pool).await?.rows_affected()
        )
    }

    /// Finds all users belonging to a specific organization.
    pub async fn get_for_organization(pool: &PgPool, organization: &Organization, user: &User) -> Result<Vec<UserWithOrganizations>, Error> {
        let users = sqlx::query!(
            r#"
            SELECT DISTINCT ON (u.id)
                u.*,
                COALESCE(
                    (
                        SELECT json_agg(json_build_object(
                            'id', o.id,
                            'name', o.name,
                            'parent_id', o.parent_id,
                            'description', o.description,
                            'slug', o.slug,
                            'created_by', o.created_by,
                            'created_at', o.created_at,
                            'updated_at', o.updated_at,
                            'deleted_at', o.deleted_at
                        ) ORDER BY o.name)
                        FROM organizations o
                        LEFT JOIN organization_users ou2 ON o.id = ou2.organization_id AND ou2.user_id = u.id AND ou2.deleted_at IS NULL
                        WHERE ou2.user_id = u.id AND ou2.deleted_at IS NULL AND o.deleted_at IS NULL
                    ),
                    '[]'
                ) AS "organizations!: serde_json::Value",
                r.id    AS "role_id?: Uuid",
                r.name  AS "role_name?: String",
                r.title AS "role_title?: String",
                ur.id    AS "user_role_id?: Uuid",
                ur.name  AS "user_role_name?: String",
                ur.title AS "user_role_title?: String"
            FROM users u
            INNER JOIN organization_users ou ON u.id = ou.user_id
            LEFT JOIN roles r              ON ou.role_id      = r.id  AND r.deleted_at  IS NULL
            LEFT JOIN organization_user_role ur ON ou.user_role_id = ur.id AND ur.deleted_at IS NULL
            WHERE u.deleted_at IS NULL AND ou.deleted_at IS NULL AND ($2 = TRUE OR ou.organization_id = $1)
            -- When showing all orgs, prefer the current-org row so role/user_role reflect the right context.
            ORDER BY u.id, (ou.organization_id = $1) DESC
            "#,
            organization.id,
            // Only internal users can see other users in their own organization.
            // internal users can also see all users when they are under imacals org.
            user.is_internal && organization.is_imacals(),
        ).fetch_all(pool).await?;

        Ok(users.into_iter().map(|row| {
            let role = match (row.role_id, row.role_name, row.role_title) {
                (Some(id), Some(name), Some(title)) => Some(RoleSummary { id, name, title }),
                _ => None,
            };
            let user_role = match (row.user_role_id, row.user_role_name, row.user_role_title) {
                (Some(id), Some(name), Some(title)) => Some(OrganizationUserRoleSummary { id, name, title }),
                _ => None,
            };
            UserWithOrganizations {
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
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    deleted_at: row.deleted_at,
                },
                organizations: serde_json::from_value(row.organizations).unwrap_or_default(),
                role,
                user_role,
            }
        }).collect())
    }

    /// Checks if an email address already exists in the database.
    ///
    /// Returns true if email exists and account is not soft-deleted, false otherwise.
    pub async fn email_exist(pool: &PgPool, needle: &str) -> Result<bool, Error>
    {
        Ok(sqlx::query!(
            "SELECT EXISTS(SELECT 1 FROM users WHERE email = LOWER($1) AND deleted_at IS NULL)",
            needle
        ).fetch_one(pool).await?.exists.unwrap())
    }

    /// Creates a new user record in the database.
    ///
    /// Takes first name, last name, email and password hash and returns the created user.
    /// Sets current_logged_in_at to NOW() for new users.
    pub async fn create(
        pool: &PgPool,
        f_name: &str,
        l_name: &str,
        email_address: &str,
        pass: &str,
    ) -> Result<User, Error> {
        Ok(
            sqlx::query_as!(
                User,
                "INSERT INTO users (first_name, last_name, email, password, current_logged_in_at) VALUES ($1, $2, LOWER($3), $4, NOW()) RETURNING *",
                f_name,
                l_name,
                email_address,
                pass
            ).fetch_one(pool).await?
        )
    }

    /// Deletes a user record (soft delete) in the database.
    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE users SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        ).execute(pool).await?.rows_affected())
    }
}