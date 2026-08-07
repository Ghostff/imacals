use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::models::domain::{CreateDomainSchema, Domain};

pub struct DomainRepository;

impl DomainRepository {
    pub async fn index(pool: &PgPool) -> Result<Vec<Domain>, Error> {
        Ok(sqlx::query_as!(
            Domain,
            "SELECT * FROM domains WHERE deleted_at IS NULL ORDER BY name ASC"
        ).fetch_all(pool).await?)
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Domain, Error> {
        Ok(sqlx::query_as!(
            Domain,
            "SELECT * FROM domains WHERE id = $1 AND deleted_at IS NULL LIMIT 1",
            id
        ).fetch_one(pool).await?)
    }

    pub async fn create(pool: &PgPool, body: &CreateDomainSchema) -> Result<Domain, Error> {
        Ok(sqlx::query_as!(
            Domain,
            r#"
            INSERT INTO domains (name, slug, country_id, state_id, city_id)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
            body.name,
            body.slug,
            body.country_id,
            body.state_id,
            body.city_id,
        ).fetch_one(pool).await?)
    }

    pub async fn update(pool: &PgPool, id: &Uuid, body: &CreateDomainSchema) -> Result<Domain, Error> {
        Ok(sqlx::query_as!(
            Domain,
            r#"
            UPDATE domains SET
                name       = $2,
                slug       = $3,
                country_id = $4,
                state_id   = $5,
                city_id    = $6,
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
            id,
            body.name,
            body.slug,
            body.country_id,
            body.state_id,
            body.city_id,
        ).fetch_one(pool).await?)
    }

    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE domains SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        ).execute(pool).await?.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    // Migration seeds the Default US domain; it must always appear.
    #[sqlx::test(migrations = "./src/migrations")]
    async fn seeded_domain_appears_in_index(pool: PgPool) {
        let rows = DomainRepository::index(&pool).await.unwrap();
        assert!(!rows.is_empty(), "at least the seeded Default US domain should be present");
        assert!(rows.iter().any(|d| d.slug == "default-us"), "Default US domain should be present");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn index_is_ordered_by_name_ascending(pool: PgPool) {
        let rows = DomainRepository::index(&pool).await.unwrap();
        let names: Vec<&str> = rows.iter().map(|d| d.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted, "domains must be returned ordered by name ASC");
    }


    // domains_location_unique is UNIQUE NULLS NOT DISTINCT over (country, state, city), and the
    // migrations already seed the country-level US row — so a test that wants its own domain has to
    // claim a location nobody holds yet.
    async fn free_us_location(pool: &PgPool) -> (Uuid, Uuid) {
        let country_id = sqlx::query_scalar!("SELECT id FROM countries WHERE iso3_code = 'USA' LIMIT 1")
            .fetch_one(pool).await.unwrap();
        let state_id = sqlx::query_scalar!(
            "SELECT s.id FROM states s
             WHERE s.country_id = $1
               AND NOT EXISTS (
                   SELECT 1 FROM domains d
                   WHERE d.state_id = s.id AND d.city_id IS NULL AND d.deleted_at IS NULL
               )
             ORDER BY s.name
             LIMIT 1",
            country_id
        ).fetch_one(pool).await.unwrap();
        (country_id, state_id)
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn created_domain_appears_in_index(pool: PgPool) {
        let (country_id, state_id) = free_us_location(&pool).await;

        let schema = CreateDomainSchema {
            name: "Test Region".into(),
            slug: "test-region".into(),
            country_id,
            state_id: Some(state_id),
            city_id: None,
        };
        DomainRepository::create(&pool, &schema).await.unwrap();

        let rows = DomainRepository::index(&pool).await.unwrap();
        assert!(rows.iter().any(|d| d.slug == "test-region"), "newly created domain must appear");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn deleted_domain_is_hidden(pool: PgPool) {
        let (country_id, state_id) = free_us_location(&pool).await;

        let schema = CreateDomainSchema {
            name: "To Delete".into(),
            slug: "to-delete".into(),
            country_id,
            state_id: Some(state_id),
            city_id: None,
        };
        let created = DomainRepository::create(&pool, &schema).await.unwrap();
        DomainRepository::delete(&pool, &created.id).await.unwrap();

        let rows = DomainRepository::index(&pool).await.unwrap();
        assert!(!rows.iter().any(|d| d.id == created.id), "deleted domain must not appear");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn find_by_id_returns_correct_domain(pool: PgPool) {
        let (country_id, state_id) = free_us_location(&pool).await;

        let schema = CreateDomainSchema {
            name: "Lookup Test".into(),
            slug: "lookup-test".into(),
            country_id,
            state_id: Some(state_id),
            city_id: None,
        };
        let created = DomainRepository::create(&pool, &schema).await.unwrap();
        let found   = DomainRepository::find_by_id(&pool, &created.id).await.unwrap();
        assert_eq!(found.id, created.id);
        assert_eq!(found.slug, "lookup-test");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn update_domain_changes_name(pool: PgPool) {
        let (country_id, state_id) = free_us_location(&pool).await;

        let schema = CreateDomainSchema {
            name: "Original Name".into(),
            slug: "original-name".into(),
            country_id,
            state_id: Some(state_id),
            city_id: None,
        };
        let created = DomainRepository::create(&pool, &schema).await.unwrap();

        let update = CreateDomainSchema {
            name: "Updated Name".into(),
            slug: "updated-name".into(),
            country_id,
            state_id: Some(state_id),
            city_id: None,
        };
        let updated = DomainRepository::update(&pool, &created.id, &update).await.unwrap();
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.slug, "updated-name");
    }
}
