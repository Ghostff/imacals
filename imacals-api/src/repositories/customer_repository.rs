use sqlx::{Error, PgPool};
use uuid::Uuid;

use crate::models::customer::Customer;

pub struct CustomerRepository;

impl CustomerRepository {
    // Find an active customer by id.
    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Customer, Error> {
        sqlx::query_as!(
            Customer,
            r#"SELECT id, organization_id, full_name, phone, email, user_id,
                      created_by, created_at, updated_at, deleted_at
               FROM customers
               WHERE id = $1 AND deleted_at IS NULL
               LIMIT 1"#,
            id
        )
        .fetch_one(pool)
        .await
    }

    // Find the customer record linked to an authenticated user. Used by the storefront to
    // resolve the acting customer's row without trusting a client-supplied id.
    pub async fn find_by_user_id(pool: &PgPool, user_id: &Uuid) -> Result<Option<Customer>, Error> {
        sqlx::query_as!(
            Customer,
            r#"SELECT id, organization_id, full_name, phone, email, user_id,
                      created_by, created_at, updated_at, deleted_at
               FROM customers
               WHERE user_id = $1 AND deleted_at IS NULL
               LIMIT 1"#,
            user_id
        )
        .fetch_optional(pool)
        .await
    }

    // Find a phone customer within an organization — the staff path for the order desk.
    pub async fn find_by_phone(
        pool: &PgPool,
        organization_id: &Uuid,
        phone: &str,
    ) -> Result<Option<Customer>, Error> {
        sqlx::query_as!(
            Customer,
            r#"SELECT id, organization_id, full_name, phone, email, user_id,
                      created_by, created_at, updated_at, deleted_at
               FROM customers
               WHERE organization_id = $1 AND phone = $2 AND deleted_at IS NULL
               LIMIT 1"#,
            organization_id,
            phone
        )
        .fetch_optional(pool)
        .await
    }

    // List customers in an organization. Used by staff to find or audit phone customers.
    pub async fn list_for_organization(
        pool: &PgPool,
        organization_id: &Uuid,
    ) -> Result<Vec<Customer>, Error> {
        sqlx::query_as!(
            Customer,
            r#"SELECT id, organization_id, full_name, phone, email, user_id,
                      created_by, created_at, updated_at, deleted_at
               FROM customers
               WHERE organization_id = $1 AND deleted_at IS NULL
               ORDER BY full_name ASC"#,
            organization_id
        )
        .fetch_all(pool)
        .await
    }

    // Create a customer row. Linking to a user_id is optional — phone-only customers have none.
    pub async fn create(
        pool: &PgPool,
        organization_id: &Uuid,
        full_name: &str,
        phone: Option<&str>,
        email: Option<&str>,
        user_id: Option<&Uuid>,
        created_by: &Uuid,
    ) -> Result<Customer, Error> {
        sqlx::query_as!(
            Customer,
            r#"INSERT INTO customers
                   (organization_id, full_name, phone, email, user_id, created_by)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, organization_id, full_name, phone, email, user_id,
                         created_by, created_at, updated_at, deleted_at"#,
            organization_id,
            full_name,
            phone,
            email,
            user_id,
            created_by
        )
        .fetch_one(pool)
        .await
    }

    // Update a customer.
    pub async fn update(pool: &PgPool, customer: &Customer) -> Result<Customer, Error> {
        sqlx::query_as!(
            Customer,
            r#"UPDATE customers
               SET full_name = $2,
                   phone     = $3,
                   email     = $4,
                   updated_at = NOW()
               WHERE id = $1 AND deleted_at IS NULL
               RETURNING id, organization_id, full_name, phone, email, user_id,
                         created_by, created_at, updated_at, deleted_at"#,
            customer.id,
            customer.full_name,
            customer.phone,
            customer.email
        )
        .fetch_one(pool)
        .await
    }

    // Soft-delete customer. The cascade trigger on the wishlists table soft-deletes their wishlists.
    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE customers SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .execute(pool)
        .await?
        .rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Error;

    async fn make_user(pool: &PgPool, email: &str) -> Uuid {
        sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, current_logged_in_at)
             VALUES ('T','T',$1,'x',NOW()) RETURNING id",
            email
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn make_org(pool: &PgPool) -> Uuid {
        sqlx::query_scalar!(
            "INSERT INTO organizations (name, slug) VALUES ('Imacals', 'imacals') RETURNING id"
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    // A new customer should come back when fetched by id.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn created_customer_can_be_fetched(pool: PgPool) {
        let org_id = make_org(&pool).await;
        let user_id = make_user(&pool, "owner@test.com").await;
        let c = CustomerRepository::create(
            &pool, &org_id, "Ada Eze", Some("08030000000"), None, None, &user_id,
        )
        .await
        .unwrap();
        let fetched = CustomerRepository::find_by_id(&pool, &c.id).await.unwrap();
        assert_eq!(fetched.full_name, "Ada Eze");
    }

    // A soft-deleted customer must not be returned by find_by_id.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn soft_deleted_customer_is_hidden(pool: PgPool) {
        let org_id = make_org(&pool).await;
        let user_id = make_user(&pool, "del@test.com").await;
        let c = CustomerRepository::create(
            &pool, &org_id, "Hidden", None, None, None, &user_id,
        )
        .await
        .unwrap();
        CustomerRepository::delete(&pool, &c.id).await.unwrap();
        let result = CustomerRepository::find_by_id(&pool, &c.id).await;
        assert!(matches!(result, Err(Error::RowNotFound)));
    }

    // Linking a customer to a user makes them findable by user_id.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn customer_linked_to_user_finds_via_user_id(pool: PgPool) {
        let org_id = make_org(&pool).await;
        let user_id = make_user(&pool, "linked@test.com").await;
        CustomerRepository::create(
            &pool, &org_id, "Online Buyer", Some("08031111111"), None, Some(&user_id), &user_id,
        )
        .await
        .unwrap();
        let found = CustomerRepository::find_by_user_id(&pool, &user_id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().full_name, "Online Buyer");
    }

    // Phone lookup must be organization-scoped — same number in a different org stays hidden.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn phone_lookup_is_org_scoped(pool: PgPool) {
        let org_id = make_org(&pool).await;
        let user_id = make_user(&pool, "phone@test.com").await;
        CustomerRepository::create(
            &pool, &org_id, "Phone Customer", Some("08032222222"), None, None, &user_id,
        )
        .await
        .unwrap();
        let other_org = sqlx::query_scalar!(
            "INSERT INTO organizations (name, slug) VALUES ('Other', 'other') RETURNING id"
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let found = CustomerRepository::find_by_phone(&pool, &other_org, "08032222222").await.unwrap();
        assert!(found.is_none());
    }
}
