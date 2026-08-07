use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

// A neighborhood or zone boundary drawn by an admin on the map.
// Properties inside the same polygon are treated as neighbors when calculating home values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polygon {
    pub id: Uuid,
    pub created_by: Uuid,
    // Stored as [{lat, lng}] JSONB so the shape can change without a schema migration.
    pub coordinates: Value,
    // Optional — links the polygon to a city for filtering, but a polygon can exist without one.
    pub city_id: Option<Uuid>,
    // Optional — the polygon zone this polygon belongs to; null when unassigned.
    pub polygon_zone_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePolygonSchema {
    pub coordinates: Value,
    pub city_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePolygonSchema {
    pub coordinates: Option<Value>,
}

// Used by PUT /polygons/:id/polygon-zone to assign or clear a polygon zone.
#[derive(Debug, Deserialize)]
pub struct AssignPolygonZoneSchema {
    pub polygon_zone_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_schema_requires_coordinates() {
        let result: Result<CreatePolygonSchema, _> = serde_json::from_str("{}");
        assert!(result.is_err(), "coordinates must be present");
    }

    #[test]
    fn create_schema_accepts_valid_coordinates() {
        let json = r#"{"coordinates": [{"lat": 25.77, "lng": -80.19}]}"#;
        let result: Result<CreatePolygonSchema, _> = serde_json::from_str(json);
        assert!(result.is_ok(), "valid coordinates should parse");
    }

    #[test]
    fn create_schema_city_id_is_optional() {
        let json = r#"{"coordinates": []}"#;
        let parsed: CreatePolygonSchema = serde_json::from_str(json).expect("should parse");
        assert!(parsed.city_id.is_none());
    }

    #[test]
    fn update_schema_coordinates_is_optional() {
        let parsed: UpdatePolygonSchema = serde_json::from_str("{}").expect("should parse");
        assert!(parsed.coordinates.is_none());
    }
}
