CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS addresses
(
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    -- Raw & canonical forms
    address_raw          TEXT NOT NULL,
    street_address       TEXT,
    -- Geospatial
    latitude             DECIMAL(11, 8),
    longitude            DECIMAL(11, 8),
    -- Street decomposition
    route_number         TEXT,
    route_prefix         TEXT,
    route                TEXT,
    route_type           TEXT,
    route_suffix         TEXT,
    unit_type            TEXT,
    unit_number          TEXT,
    intersection         TEXT,
    -- Administrative hierarchy
    country_id           UUID REFERENCES countries(id) ON DELETE RESTRICT NOT NULL,
    state_id             UUID REFERENCES states(id) ON DELETE RESTRICT NULL,
    county               TEXT,
    locality             TEXT,
    sublocality          TEXT,
    subdivision          TEXT,
    neighborhood         TEXT,
    school_district      TEXT,
    zip                  TEXT,

    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ NULL
);

-- =========================
-- Indexes
-- =========================

-- FK lookups for joins on administrative hierarchy.
CREATE INDEX IF NOT EXISTS addresses_country_id_index
    ON addresses (country_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS addresses_state_id_index
    ON addresses (state_id)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering on common listings.
CREATE INDEX IF NOT EXISTS addresses_deleted_at_index
    ON addresses (deleted_at);

-- Postal code lookups for search / autocomplete.
CREATE INDEX IF NOT EXISTS addresses_zip_index
    ON addresses (zip)
    WHERE deleted_at IS NULL;

-- Geospatial proximity queries by (lat, lng) bounding box.
CREATE INDEX IF NOT EXISTS addresses_lat_lng_index
    ON addresses (latitude, longitude)
    WHERE deleted_at IS NULL;
