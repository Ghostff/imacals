use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// A single directed edge in the neighbor graph; both (A→B) and (B→A) are stored so each
// polygon can query its own neighbors without a UNION.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonNeighbor {
    pub polygon_id: Uuid,
    pub neighbor_polygon_id: Uuid,
}

// What the caller sends to create a neighbor link between two polygons.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateNeighborSchema {
    pub polygon_id: Uuid,
    pub neighbor_polygon_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    // A valid payload with two different UUIDs should deserialize cleanly.
    #[test]
    fn create_schema_parses_valid_payload() {
        let json = r#"{"polygon_id":"00000000-0000-0000-0000-000000000001","neighbor_polygon_id":"00000000-0000-0000-0000-000000000002"}"#;
        let result: Result<CreateNeighborSchema, _> = serde_json::from_str(json);
        assert!(result.is_ok());
    }

    // A payload missing neighbor_polygon_id must fail.
    #[test]
    fn create_schema_rejects_missing_neighbor_id() {
        let json = r#"{"polygon_id":"00000000-0000-0000-0000-000000000001"}"#;
        let result: Result<CreateNeighborSchema, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
