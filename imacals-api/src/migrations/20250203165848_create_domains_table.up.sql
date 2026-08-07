CREATE TABLE domains (
    id         UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    name       TEXT        NOT NULL,
    -- Stable key used for lookups (e.g. 'default-us'). Never changes after creation.
    slug       TEXT        NOT NULL,
    country_id UUID        NOT NULL REFERENCES countries(id),
    state_id   UUID        REFERENCES states(id),
    city_id    UUID        REFERENCES cities(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT domains_slug_unique     UNIQUE (slug),
    CONSTRAINT domains_location_unique UNIQUE NULLS NOT DISTINCT (country_id, state_id, city_id)
);

CREATE INDEX IF NOT EXISTS domains_country_id_idx ON domains (country_id);
CREATE INDEX IF NOT EXISTS domains_state_id_idx   ON domains (state_id);
CREATE INDEX IF NOT EXISTS domains_city_id_idx    ON domains (city_id);

INSERT INTO domains (name, slug, country_id)
SELECT 'Default US', 'default-us', id FROM countries WHERE iso3_code = 'USA'
LIMIT 1;

INSERT INTO domains (name, slug, country_id, state_id, city_id)
SELECT
    'San Antonio, TX',
    'san-antonio-tx',
    c.id,
    s.id,
    ci.id
FROM countries c
JOIN states  s  ON s.code = 'TX' AND s.country_id = c.id
JOIN cities  ci ON ci.name ILIKE 'San Antonio' AND ci.state_id = s.id
WHERE c.iso3_code = 'USA'
LIMIT 1;
