use serde_json::Value;
use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::models::polygon::Polygon;

pub struct PolygonRepository;

impl PolygonRepository {
    pub async fn index(pool: &PgPool) -> Result<Vec<Polygon>, Error> {
        Ok(sqlx::query_as!(
            Polygon,
            r#"SELECT id, created_by,
                      coordinates AS "coordinates: Value",
                      city_id, polygon_zone_id, created_at, updated_at, deleted_at
               FROM polygons
               WHERE deleted_at IS NULL
               ORDER BY created_at DESC"#
        ).fetch_all(pool).await?)
    }

    pub async fn find_by_id(pool: &PgPool, id: &Uuid) -> Result<Polygon, Error> {
        Ok(sqlx::query_as!(
            Polygon,
            r#"SELECT id, created_by,
                      coordinates AS "coordinates: Value",
                      city_id, polygon_zone_id, created_at, updated_at, deleted_at
               FROM polygons
               WHERE id = $1 AND deleted_at IS NULL
               LIMIT 1"#,
            id
        ).fetch_one(pool).await?)
    }

    pub async fn create(
        pool: &PgPool,
        created_by: &Uuid,
        coordinates: &Value,
        city_id: Option<&Uuid>,
    ) -> Result<Polygon, Error> {
        Ok(sqlx::query_as!(
            Polygon,
            r#"INSERT INTO polygons (created_by, coordinates, city_id)
               VALUES ($1, $2, $3)
               RETURNING id, created_by,
                         coordinates AS "coordinates: Value",
                         city_id, polygon_zone_id, created_at, updated_at, deleted_at"#,
            created_by,
            coordinates as &Value,
            city_id
        ).fetch_one(pool).await?)
    }

    pub async fn update(
        pool: &PgPool,
        id: &Uuid,
        coordinates: &Value,
    ) -> Result<Polygon, Error> {
        Ok(sqlx::query_as!(
            Polygon,
            r#"UPDATE polygons
               SET coordinates = $2,
                   updated_at  = NOW()
               WHERE id = $1 AND deleted_at IS NULL
               RETURNING id, created_by,
                         coordinates AS "coordinates: Value",
                         city_id, polygon_zone_id, created_at, updated_at, deleted_at"#,
            id,
            coordinates as &Value
        ).fetch_one(pool).await?)
    }

    // Assigns (or clears) a polygon zone; polygon_zone_id = None removes the assignment.
    pub async fn assign_polygon_zone(pool: &PgPool, id: &Uuid, polygon_zone_id: Option<&Uuid>) -> Result<Polygon, Error> {
        Ok(sqlx::query_as!(
            Polygon,
            r#"UPDATE polygons
               SET polygon_zone_id = $2,
                   updated_at      = NOW()
               WHERE id = $1 AND deleted_at IS NULL
               RETURNING id, created_by,
                         coordinates AS "coordinates: Value",
                         city_id, polygon_zone_id, created_at, updated_at, deleted_at"#,
            id,
            polygon_zone_id
        ).fetch_one(pool).await?)
    }

    // Soft-delete: preserving the row lets us audit or recover shapes after deletion.
    pub async fn delete(pool: &PgPool, id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "UPDATE polygons SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
            id
        ).execute(pool).await?.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    async fn seed_user(pool: &PgPool) -> Uuid {
        sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, current_logged_in_at)
             VALUES ('Test','User','poly_test@test.com','x',NOW()) RETURNING id"
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn created_polygon_appears_in_index(pool: PgPool) {
        let uid = seed_user(&pool).await;
        PolygonRepository::create(&pool, &uid, &json!([{"lat": 1.0, "lng": 2.0}]), None)
            .await
            .unwrap();

        let rows = PolygonRepository::index(&pool).await.unwrap();
        assert_eq!(rows.len(), 1, "one polygon should be returned");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn deleted_polygon_is_hidden_from_index(pool: PgPool) {
        let uid = seed_user(&pool).await;
        let p = PolygonRepository::create(&pool, &uid, &json!([]), None)
            .await
            .unwrap();

        let affected = PolygonRepository::delete(&pool, &p.id).await.unwrap();
        assert_eq!(affected, 1, "one row should be affected");

        let rows = PolygonRepository::index(&pool).await.unwrap();
        assert!(rows.is_empty(), "deleted polygon must not appear in the list");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn double_delete_returns_zero(pool: PgPool) {
        let uid = seed_user(&pool).await;
        let p = PolygonRepository::create(&pool, &uid, &json!([]), None)
            .await
            .unwrap();

        PolygonRepository::delete(&pool, &p.id).await.unwrap();
        let affected = PolygonRepository::delete(&pool, &p.id).await.unwrap();
        assert_eq!(affected, 0, "already-deleted polygon should affect zero rows");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn update_replaces_coordinates(pool: PgPool) {
        let uid = seed_user(&pool).await;
        let p = PolygonRepository::create(&pool, &uid, &json!([{"lat": 1.0, "lng": 1.0}]), None)
            .await
            .unwrap();

        let new_coords = json!([{"lat": 9.0, "lng": 9.0}]);
        let updated = PolygonRepository::update(&pool, &p.id, &new_coords)
            .await
            .unwrap();

        assert_eq!(updated.coordinates, new_coords, "coordinates should be replaced");
    }
}
