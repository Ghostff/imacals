CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "countries"
(
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    -- Core country info
    name          TEXT NOT NULL,
    iso2_code     CHAR(2) NOT NULL,
    iso3_code     CHAR(3) NOT NULL,
    numeric_code  CHAR(3),
    phone_code    TEXT,
    currency_code CHAR(3),
    region        TEXT,
    subregion     TEXT,

    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =========================
-- Indexes
-- =========================

-- Uniqueness constraints
CREATE UNIQUE INDEX IF NOT EXISTS countries_iso2_code_uindex ON countries (iso2_code);

CREATE UNIQUE INDEX IF NOT EXISTS countries_iso3_code_uindex ON countries (iso3_code);

-- Search & filtering
CREATE INDEX IF NOT EXISTS countries_name_index
    ON countries (name);

CREATE INDEX IF NOT EXISTS countries_region_index
    ON countries (region);

INSERT INTO countries (name, iso2_code, iso3_code, numeric_code, phone_code, currency_code, region, subregion)
VALUES
    ('United States', 'US', 'USA', '840', '+1', 'USD','Americas','Northern America'),
    ('Canada','CA','CAN','124','+1','CAD','Americas','Northern America')
ON CONFLICT (iso2_code) DO UPDATE SET updated_at = NOW();
