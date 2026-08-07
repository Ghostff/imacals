use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// A Country is a top-level geographic region stored as reference data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Country {
    pub id: Uuid,
    pub name: String,
    pub iso2_code: String,
    pub iso3_code: String,
    pub numeric_code: Option<String>,
    pub phone_code: Option<String>,
    pub currency_code: Option<String>,
    pub region: Option<String>,
    pub subregion: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// A State (or province) belongs to a country.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub id: Uuid,
    pub country_id: Uuid,
    pub name: String,
    pub code: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// A City belongs to a state and carries the coordinates used to centre the map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct City {
    pub id: Uuid,
    pub state_id: Uuid,
    pub name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
