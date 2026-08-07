use sqlx::{Error, PgPool};
use uuid::Uuid;

use crate::models::domain_system_user::{CreateDomainSystemUserSchema, DomainSystemUserView};

pub struct DomainSystemUserRepository;

impl DomainSystemUserRepository {
    pub async fn index(pool: &PgPool) -> Result<Vec<DomainSystemUserView>, Error> {
        Ok(sqlx::query_as!(
            DomainSystemUserView,
            r#"
            SELECT
                dsu.id,
                dsu.domain_id,
                d.name             AS domain_name,
                dsu.user_id,
                u.first_name       AS user_first_name,
                u.last_name        AS user_last_name,
                u.email            AS user_email,
                dsu.user_role_id,
                ur.name            AS role_name,
                ur.title           AS role_title,
                dsu.created_by,
                dsu.created_at,
                dsu.updated_at,
                dsu.deleted_at
            FROM domain_system_users dsu
            JOIN domains                d  ON d.id  = dsu.domain_id
            JOIN users                  u  ON u.id  = dsu.user_id
            JOIN organization_user_role ur ON ur.id = dsu.user_role_id
            WHERE dsu.deleted_at IS NULL
            ORDER BY d.name ASC, ur.title ASC
            "#
        ).fetch_all(pool).await?)
    }

    // Replaces any existing active assignment for (domain_id, user_role_id) with the new user.
    pub async fn upsert(
        pool: &PgPool,
        body: &CreateDomainSystemUserSchema,
        created_by: &Uuid,
    ) -> Result<DomainSystemUserView, Error> {
        sqlx::query!(
            "UPDATE domain_system_users SET deleted_at = NOW(), updated_at = NOW()
             WHERE domain_id = $1 AND user_role_id = $2 AND deleted_at IS NULL",
            body.domain_id,
            body.user_role_id,
        ).execute(pool).await?;

        sqlx::query_as!(
            DomainSystemUserView,
            r#"
            INSERT INTO domain_system_users (domain_id, user_id, user_role_id, created_by)
            VALUES ($1, $2, $3, $4)
            RETURNING
                id,
                domain_id,
                (SELECT name  FROM domains                WHERE id = $1) AS "domain_name!",
                user_id,
                (SELECT first_name FROM users             WHERE id = $2) AS "user_first_name!",
                (SELECT last_name  FROM users             WHERE id = $2) AS "user_last_name!",
                (SELECT email      FROM users             WHERE id = $2) AS "user_email!",
                user_role_id,
                (SELECT name  FROM organization_user_role WHERE id = $3) AS "role_name!",
                (SELECT title FROM organization_user_role WHERE id = $3) AS "role_title!",
                created_by,
                created_at,
                updated_at,
                deleted_at
            "#,
            body.domain_id,
            body.user_id,
            body.user_role_id,
            created_by,
        ).fetch_one(pool).await
    }

    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE domain_system_users SET deleted_at = NOW(), updated_at = NOW()
             WHERE id = $1 AND deleted_at IS NULL",
            id
        ).execute(pool).await?.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    use uuid::Uuid;

    async fn seed_user(pool: &PgPool, email: &str) -> Uuid {
        sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, current_logged_in_at)
             VALUES ('T','T',$1,'x',NOW()) RETURNING id",
            email
        ).fetch_one(pool).await.unwrap()
    }

    async fn default_domain(pool: &PgPool) -> Uuid {
        sqlx::query_scalar!("SELECT id FROM domains WHERE slug = 'default-us' LIMIT 1")
            .fetch_one(pool).await.unwrap()
    }

    // Fetches the id of an eligible role (broker) for tests.
    async fn broker_role(pool: &PgPool) -> Uuid {
        sqlx::query_scalar!(
            "SELECT id FROM organization_user_role
             WHERE name = 'broker' AND system_user_eligible = TRUE AND organization_id IS NULL
             LIMIT 1"
        ).fetch_one(pool).await.unwrap()
    }

    async fn realtor_role(pool: &PgPool) -> Uuid {
        sqlx::query_scalar!(
            "SELECT id FROM organization_user_role
             WHERE name = 'realtor' AND system_user_eligible = TRUE AND organization_id IS NULL
             LIMIT 1"
        ).fetch_one(pool).await.unwrap()
    }

    // A fresh upsert should appear in the index.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn upserted_record_appears_in_index(pool: PgPool) {
        let user_id      = seed_user(&pool, "sys1@test.com").await;
        let domain_id    = default_domain(&pool).await;
        let user_role_id = broker_role(&pool).await;

        let body = CreateDomainSystemUserSchema { domain_id, user_id, user_role_id };
        DomainSystemUserRepository::upsert(&pool, &body, &user_id).await.unwrap();

        let rows = DomainSystemUserRepository::index(&pool).await.unwrap();
        assert!(rows.iter().any(|r| r.user_id == user_id && r.role_name == "broker"));
    }

    // A second upsert for the same domain+role replaces the first.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn upsert_replaces_existing_assignment(pool: PgPool) {
        let user_a       = seed_user(&pool, "sys_a@test.com").await;
        let user_b       = seed_user(&pool, "sys_b@test.com").await;
        let domain_id    = default_domain(&pool).await;
        let user_role_id = realtor_role(&pool).await;

        DomainSystemUserRepository::upsert(
            &pool,
            &CreateDomainSystemUserSchema { domain_id, user_id: user_a, user_role_id },
            &user_a,
        ).await.unwrap();

        DomainSystemUserRepository::upsert(
            &pool,
            &CreateDomainSystemUserSchema { domain_id, user_id: user_b, user_role_id },
            &user_b,
        ).await.unwrap();

        let rows = DomainSystemUserRepository::index(&pool).await.unwrap();
        let realtors: Vec<_> = rows.iter().filter(|r| r.role_name == "realtor").collect();
        assert_eq!(realtors.len(), 1, "only one realtor per domain");
        assert_eq!(realtors[0].user_id, user_b, "second upsert should win");
    }

    // Soft-deleted record must not appear in the index.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn deleted_record_is_hidden(pool: PgPool) {
        let user_id      = seed_user(&pool, "sys2@test.com").await;
        let domain_id    = default_domain(&pool).await;
        let user_role_id = broker_role(&pool).await;

        let created = DomainSystemUserRepository::upsert(
            &pool,
            &CreateDomainSystemUserSchema { domain_id, user_id, user_role_id },
            &user_id,
        ).await.unwrap();

        DomainSystemUserRepository::delete(&pool, &created.id).await.unwrap();

        let rows = DomainSystemUserRepository::index(&pool).await.unwrap();
        assert!(!rows.iter().any(|r| r.id == created.id));
    }
}
