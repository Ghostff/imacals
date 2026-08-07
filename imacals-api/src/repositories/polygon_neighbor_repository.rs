use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::models::polygon_neighbor::PolygonNeighbor;

pub struct PolygonNeighborRepository;

impl PolygonNeighborRepository {
    // Returns one row per unique pair (polygon_id < neighbor_polygon_id avoids returning both directions).
    pub async fn index(pool: &PgPool) -> Result<Vec<PolygonNeighbor>, Error> {
        Ok(sqlx::query_as!(
            PolygonNeighbor,
            "SELECT polygon_id, neighbor_polygon_id
             FROM polygon_neighbors
             WHERE polygon_id < neighbor_polygon_id
             ORDER BY polygon_id"
        ).fetch_all(pool).await?)
    }

    // Inserts both directions so the link is bidirectional; ignores duplicates.
    pub async fn create(pool: &PgPool, polygon_id: &Uuid, neighbor_id: &Uuid) -> Result<(), Error> {
        sqlx::query!(
            "INSERT INTO polygon_neighbors (polygon_id, neighbor_polygon_id)
             VALUES ($1, $2), ($2, $1)
             ON CONFLICT DO NOTHING",
            polygon_id,
            neighbor_id
        ).execute(pool).await?;
        Ok(())
    }

    // Removes the link in both directions; returns how many rows were actually deleted.
    pub async fn delete(pool: &PgPool, polygon_id: &Uuid, neighbor_id: &Uuid) -> Result<u64, Error> {
        Ok(sqlx::query!(
            "DELETE FROM polygon_neighbors
             WHERE (polygon_id = $1 AND neighbor_polygon_id = $2)
                OR (polygon_id = $2 AND neighbor_polygon_id = $1)",
            polygon_id,
            neighbor_id
        ).execute(pool).await?.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::repositories::polygon_repository::PolygonRepository;

    async fn seed_user(pool: &PgPool) -> Uuid {
        sqlx::query_scalar!(
            "INSERT INTO users (first_name, last_name, email, password, current_logged_in_at)
             VALUES ('T','T',$1,'x',NOW()) RETURNING id",
            &format!("neighbor_test_{}@test.com", uuid::Uuid::new_v4())
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }

    async fn seed_polygon(pool: &PgPool, uid: &Uuid) -> Uuid {
        PolygonRepository::create(pool, uid, &json!([{"lat":1.0,"lng":1.0}]), None)
            .await
            .unwrap()
            .id
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn create_and_index_neighbor(pool: PgPool) {
        let uid = seed_user(&pool).await;
        let a = seed_polygon(&pool, &uid).await;
        let b = seed_polygon(&pool, &uid).await;

        PolygonNeighborRepository::create(&pool, &a, &b).await.unwrap();

        let rows = PolygonNeighborRepository::index(&pool).await.unwrap();
        assert_eq!(rows.len(), 1, "index returns exactly one row per pair");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn duplicate_create_is_idempotent(pool: PgPool) {
        let uid = seed_user(&pool).await;
        let a = seed_polygon(&pool, &uid).await;
        let b = seed_polygon(&pool, &uid).await;

        PolygonNeighborRepository::create(&pool, &a, &b).await.unwrap();
        PolygonNeighborRepository::create(&pool, &a, &b).await.unwrap();

        let rows = PolygonNeighborRepository::index(&pool).await.unwrap();
        assert_eq!(rows.len(), 1, "duplicate inserts must not create extra rows");
    }

    #[sqlx::test(migrations = "./src/migrations")]
    async fn delete_removes_both_directions(pool: PgPool) {
        let uid = seed_user(&pool).await;
        let a = seed_polygon(&pool, &uid).await;
        let b = seed_polygon(&pool, &uid).await;

        PolygonNeighborRepository::create(&pool, &a, &b).await.unwrap();
        let affected = PolygonNeighborRepository::delete(&pool, &a, &b).await.unwrap();

        assert_eq!(affected, 2, "both directions should be removed");
        let rows = PolygonNeighborRepository::index(&pool).await.unwrap();
        assert!(rows.is_empty(), "no neighbors should remain after delete");
    }
}
