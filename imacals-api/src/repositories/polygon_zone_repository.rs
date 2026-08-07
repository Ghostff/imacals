use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::models::polygon_zone::{PolygonZone, CreatePolygonZoneSchema, UpdatePolygonZoneSchema};

// PolygonZoneRepository is the only place that talks to the polygon_zones table.
pub struct PolygonZoneRepository;

impl PolygonZoneRepository {
    pub async fn index(pool: &PgPool) -> Result<Vec<PolygonZone>, Error> {
        Ok(sqlx::query_as!(
            PolygonZone,
            "SELECT * FROM polygon_zones WHERE deleted_at IS NULL ORDER BY name ASC"
        ).fetch_all(pool).await?)
    }

    pub async fn create(pool: &PgPool, schema: &CreatePolygonZoneSchema, created_by: &Uuid) -> Result<PolygonZone, Error> {
        Ok(sqlx::query_as!(
            PolygonZone,
            "INSERT INTO polygon_zones (name, color, created_by) VALUES ($1, $2, $3) RETURNING *",
            schema.name,
            schema.color,
            created_by
        ).fetch_one(pool).await?)
    }

    pub async fn update(pool: &PgPool, id: &Uuid, schema: &UpdatePolygonZoneSchema) -> Result<PolygonZone, Error> {
        Ok(sqlx::query_as!(
            PolygonZone,
            "UPDATE polygon_zones SET name = $1, color = $2, updated_at = NOW() WHERE id = $3 AND deleted_at IS NULL RETURNING *",
            schema.name,
            schema.color,
            id
        ).fetch_one(pool).await?)
    }
}
