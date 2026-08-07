-- Up migration: create_cities_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "cities"
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    state_id    UUID REFERENCES states(id) ON DELETE RESTRICT NOT NULL,
    name        TEXT NOT NULL,
    latitude    NUMERIC(9,6),
    longitude   NUMERIC(9,6),

    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =========================
-- Indexes
-- =========================

CREATE INDEX IF NOT EXISTS cities_state_id_index
    ON cities (state_id);

CREATE INDEX IF NOT EXISTS cities_name_index
    ON cities (name);

CREATE UNIQUE INDEX IF NOT EXISTS cities_state_name_uindex
    ON cities (state_id, name);


-- =========================
-- Seeders: Texas cities
-- =========================
WITH tx AS (
    SELECT s.id AS state_id
    FROM states s
    JOIN countries c ON c.id = s.country_id
    WHERE c.iso3_code = 'USA' AND s.code = 'TX'
    LIMIT 1
)
INSERT INTO cities (state_id, name, latitude, longitude)
SELECT tx.state_id, c.name, c.latitude, c.longitude
FROM tx CROSS JOIN (VALUES
    ('Houston',         29.760427,  -95.369804),
    ('San Antonio',     29.424122,  -98.493628),
    ('Dallas',          32.776664,  -96.796988),
    ('Austin',          30.267153,  -97.743057),
    ('Fort Worth',      32.725409,  -97.320862),
    ('El Paso',         31.761878, -106.485022),
    ('Arlington',       32.735687,  -97.108066),
    ('Corpus Christi',  27.800583,  -97.396381),
    ('Plano',           33.019843,  -96.698886),
    ('Laredo',          27.506407,  -99.507445),
    ('Lubbock',         33.577863, -101.855166),
    ('Garland',         32.912624,  -96.638833),
    ('Irving',          32.814018,  -96.948895),
    ('Amarillo',        35.221997, -101.831297),
    ('Grand Prairie',   32.745964,  -96.997793),
    ('Brownsville',     25.901747,  -97.497484),
    ('McKinney',        33.197540,  -96.615556),
    ('Frisco',          33.150674,  -96.823610),
    ('Pasadena',        29.691094,  -95.209099),
    ('Killeen',         31.117119,  -97.727796),
    ('McAllen',         26.203407,  -98.230012),
    ('Mesquite',        32.766838,  -96.599076),
    ('Waco',            31.549333,  -97.146988),
    ('Carrollton',      32.954200,  -96.890023),
    ('Beaumont',        30.080174,  -94.126557),
    ('Abilene',         32.448736,  -99.733145),
    ('Denton',          33.214841,  -97.133068),
    ('Odessa',          31.845683, -102.367589),
    ('Midland',         31.997345, -102.077915),
    ('Round Rock',      30.508211,  -97.678896),
    ('Richardson',      32.948200,  -96.729900),
    ('Pearland',        29.563567,  -95.286214),
    ('College Station', 30.627977,  -96.334407),
    ('Lewisville',      33.046233,  -96.994174),
    ('Tyler',           32.351487,  -95.301460),
    ('League City',     29.507723,  -95.094907),
    ('Wichita Falls',   33.913708,  -98.493228),
    ('San Angelo',      31.463765, -100.437401),
    ('Edinburg',        26.301891,  -98.163338),
    ('Allen',           33.103144,  -96.670526)
) AS c(name, latitude, longitude)
ON CONFLICT DO NOTHING;
