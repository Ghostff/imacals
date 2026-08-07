use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

// A named color group that an admin can paint onto polygons to indicate a neighbourhood zone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolygonZone {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePolygonZoneSchema {
    #[validate(length(min = 1, max = 100, message = "Name must be between 1 and 100 characters"))]
    pub name: String,
    // Must be a 7-character CSS hex color (#RRGGBB). The frontend supplies one of the fixed palette swatches.
    #[validate(custom(function = "validate_hex_color"))]
    pub color: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePolygonZoneSchema {
    #[validate(length(min = 1, max = 100, message = "Name must be between 1 and 100 characters"))]
    pub name: String,
    #[validate(custom(function = "validate_hex_color"))]
    pub color: String,
}

fn validate_hex_color(color: &str) -> Result<(), ValidationError> {
    if color.len() == 7 && color.starts_with('#') && color[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_hex_color"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[test]
    fn rejects_empty_name() {
        let s = CreatePolygonZoneSchema { name: "".into(), color: "#EF4444".into() };
        assert!(s.validate().is_err());
    }

    #[test]
    fn accepts_valid_payload() {
        let s = CreatePolygonZoneSchema { name: "Downtown".into(), color: "#EF4444".into() };
        assert!(s.validate().is_ok());
    }

    #[test]
    fn rejects_invalid_color() {
        let s = CreatePolygonZoneSchema { name: "Zone".into(), color: "red".into() };
        assert!(s.validate().is_err());
    }

    #[test]
    fn rejects_short_hex() {
        let s = CreatePolygonZoneSchema { name: "Zone".into(), color: "#F00".into() };
        assert!(s.validate().is_err());
    }
}
