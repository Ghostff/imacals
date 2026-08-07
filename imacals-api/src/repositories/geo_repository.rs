use sqlx::{Error, PgPool};
use uuid::Uuid;
use crate::models::geo::{Country, State, City};

// GeoRepository is the only place that reads the countries/states/cities tables.
pub struct GeoRepository;

impl GeoRepository {
    pub async fn list_countries(pool: &PgPool) -> Result<Vec<Country>, Error> {
        Ok(sqlx::query_as!(
            Country,
            "SELECT * FROM countries ORDER BY name"
        ).fetch_all(pool).await?)
    }

    pub async fn list_states_by_country(pool: &PgPool, country_id: &Uuid) -> Result<Vec<State>, Error> {
        Ok(sqlx::query_as!(
            State,
            r#"SELECT
                id, country_id, name, code,
                CAST(latitude  AS FLOAT8) AS latitude,
                CAST(longitude AS FLOAT8) AS longitude,
                created_at, updated_at
            FROM states
            WHERE country_id = $1
            ORDER BY name"#,
            country_id
        ).fetch_all(pool).await?)
    }

    pub async fn list_cities_by_state(pool: &PgPool, state_id: &Uuid) -> Result<Vec<City>, Error> {
        Ok(sqlx::query_as!(
            City,
            r#"SELECT
                id, state_id, name,
                CAST(latitude  AS FLOAT8) AS latitude,
                CAST(longitude AS FLOAT8) AS longitude,
                created_at, updated_at
            FROM cities
            WHERE state_id = $1
            ORDER BY name"#,
            state_id
        ).fetch_all(pool).await?)
    }
}
