-- =========================================================================
-- Migration: 20250203165754_create_soft_delete_cascade_functions.up.sql
-- =========================================================================
-- Up migration: create_soft_delete_cascade_functions
-- Reusable trigger functions for cascading soft-deletes.
-- Attach these to any table that has children using parent_id or owner_type/owner_id.

-- ─────────────────────────────────────────────────────────────────────────────
-- 1. parent_id cascade (self-referential tables, e.g. organizations)
--    When a row's deleted_at transitions NULL → value, soft-delete all rows
--    in the SAME table whose parent_id = NEW.id.
-- ─────────────────────────────────────────────────────────────────────────────
CREATE OR REPLACE FUNCTION soft_delete_cascade_by_parent_id()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        EXECUTE format(
            'UPDATE %I SET deleted_at = $1 WHERE parent_id = $2 AND deleted_at IS NULL',
            TG_TABLE_NAME
        ) USING NEW.deleted_at, NEW.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ─────────────────────────────────────────────────────────────────────────────
-- 2. owner_type / owner_id cascade (polymorphic ownership, e.g. attributes)
--    When a row's deleted_at transitions NULL → value, soft-delete all rows
--    in the target table whose owner_type = TG_TABLE_NAME AND owner_id = NEW.id.
--    Pass the target table name via TG_ARGV[0].
--    Example attachment:
--      CREATE TRIGGER trg_soft_delete_attributes_on_facility_units_delete
--          AFTER UPDATE OF deleted_at ON facility_units
--          FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_owner('attributes');
-- ─────────────────────────────────────────────────────────────────────────────
CREATE OR REPLACE FUNCTION soft_delete_cascade_by_owner()
RETURNS TRIGGER AS $$
DECLARE
    target_table TEXT := TG_ARGV[0];
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        EXECUTE format(
            'UPDATE %I SET deleted_at = $1 WHERE owner_type = $2 AND owner_id = $3 AND deleted_at IS NULL',
            target_table
        ) USING NEW.deleted_at, TG_TABLE_NAME, NEW.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ─────────────────────────────────────────────────────────────────────────────
-- 3. Direct FK cascade (child table references parent via a named FK column)
--    When a row's deleted_at transitions NULL → value, soft-delete all rows
--    in the target table whose <fk_column> = NEW.id.
--    Pass target table name as TG_ARGV[0] and FK column name as TG_ARGV[1].
--    Example attachment:
--      CREATE TRIGGER trg_soft_delete_facility_units_on_facilities_delete
--          AFTER UPDATE OF deleted_at ON facilities
--          FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_fk('facility_units', 'facility_id');
-- ─────────────────────────────────────────────────────────────────────────────
CREATE OR REPLACE FUNCTION soft_delete_cascade_by_fk()
RETURNS TRIGGER AS $$
DECLARE
    target_table TEXT := TG_ARGV[0];
    fk_column    TEXT := TG_ARGV[1];
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        EXECUTE format(
            'UPDATE %I SET deleted_at = $1 WHERE %I = $2 AND deleted_at IS NULL',
            target_table, fk_column
        ) USING NEW.deleted_at, NEW.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- =========================================================================
-- Migration: 20250203165755_create_countries_table.up.sql
-- =========================================================================
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


-- =========================================================================
-- Migration: 20250203165845_create_states_table.up.sql
-- =========================================================================
-- Up migration: create_states_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "states"
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    country_id  UUID REFERENCES countries(id) ON DELETE RESTRICT NOT NULL,
    name        TEXT NOT NULL,
    code        TEXT NOT NULL, -- e.g. CA, NY, TX
    latitude    NUMERIC(9,6),
    longitude   NUMERIC(9,6),

    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =========================
-- Indexes
-- =========================

CREATE INDEX IF NOT EXISTS states_country_id_index
    ON states (country_id);

CREATE INDEX IF NOT EXISTS states_name_index
    ON states (name);

CREATE UNIQUE INDEX IF NOT EXISTS states_country_code_uindex ON states (country_id, code);


-- =========================
-- Seeders
-- =========================
WITH us AS (
    SELECT id AS country_id FROM countries WHERE iso3_code = 'USA' LIMIT 1
)
INSERT INTO states (country_id, name, code, latitude, longitude)
SELECT us.country_id, s.name, s.code, s.latitude, s.longitude
FROM us CROSS JOIN (VALUES
    ('Alabama',        'AL',  32.806671,  -86.791130),
    ('Alaska',         'AK',  61.370716, -152.404419),
    ('Arizona',        'AZ',  33.729759, -111.431221),
    ('Arkansas',       'AR',  34.969704,  -92.373123),
    ('California',     'CA',  36.116203, -119.681564),
    ('Colorado',       'CO',  39.059811, -105.311104),
    ('Connecticut',    'CT',  41.597782,  -72.755371),
    ('Delaware',       'DE',  39.318523,  -75.507141),
    ('Florida',        'FL',  27.766279,  -81.686783),
    ('Georgia',        'GA',  33.040619,  -83.643074),
    ('Hawaii',         'HI',  21.094318, -157.498337),
    ('Idaho',          'ID',  44.240459, -114.478828),
    ('Illinois',       'IL',  40.349457,  -88.986137),
    ('Indiana',        'IN',  39.849426,  -86.258278),
    ('Iowa',           'IA',  42.011539,  -93.210526),
    ('Kansas',         'KS',  38.526600,  -96.726486),
    ('Kentucky',       'KY',  37.668140,  -84.670067),
    ('Louisiana',      'LA',  31.169960,  -91.867805),
    ('Maine',          'ME',  44.693947,  -69.381927),
    ('Maryland',       'MD',  39.063946,  -76.802101),
    ('Massachusetts',  'MA',  42.230171,  -71.530106),
    ('Michigan',       'MI',  43.326618,  -84.536095),
    ('Minnesota',      'MN',  45.694454,  -93.900192),
    ('Mississippi',    'MS',  32.741646,  -89.678696),
    ('Missouri',       'MO',  38.456085,  -92.288368),
    ('Montana',        'MT',  46.921925, -110.454353),
    ('Nebraska',       'NE',  41.125370,  -98.268082),
    ('Nevada',         'NV',  38.313515, -117.055374),
    ('New Hampshire',  'NH',  43.452492,  -71.563896),
    ('New Jersey',     'NJ',  40.298904,  -74.521011),
    ('New Mexico',     'NM',  34.840515, -106.248482),
    ('New York',       'NY',  42.165726,  -74.948051),
    ('North Carolina', 'NC',  35.630066,  -79.806419),
    ('North Dakota',   'ND',  47.528912,  -99.784012),
    ('Ohio',           'OH',  40.388783,  -82.764915),
    ('Oklahoma',       'OK',  35.565342,  -96.928917),
    ('Oregon',         'OR',  44.572021, -122.070938),
    ('Pennsylvania',   'PA',  40.590752,  -77.209755),
    ('Rhode Island',   'RI',  41.680893,  -71.511780),
    ('South Carolina', 'SC',  33.856892,  -80.945007),
    ('South Dakota',   'SD',  44.299782,  -99.438828),
    ('Tennessee',      'TN',  35.747845,  -86.692345),
    ('Texas',          'TX',  31.054487,  -97.563461),
    ('Utah',           'UT',  40.150032, -111.862434),
    ('Vermont',        'VT',  44.045876,  -72.710686),
    ('Virginia',       'VA',  37.769337,  -78.169968),
    ('Washington',     'WA',  47.400902, -121.490494),
    ('West Virginia',  'WV',  38.491226,  -80.954453),
    ('Wisconsin',      'WI',  44.268543,  -89.616508),
    ('Wyoming',        'WY',  42.755966, -107.302490)
) AS s(name, code, latitude, longitude)
ON CONFLICT DO NOTHING;

WITH ca AS (
    SELECT id AS country_id FROM countries WHERE iso3_code = 'CAN' LIMIT 1
)
INSERT INTO states (country_id, name, code, latitude, longitude)
SELECT ca.country_id, s.name, s.code, s.latitude, s.longitude
FROM ca CROSS JOIN (VALUES
    ('Alberta',                    'AB',  53.933271, -116.576503),
    ('British Columbia',           'BC',  53.726669, -127.647621),
    ('Manitoba',                   'MB',  53.760860,  -98.813873),
    ('New Brunswick',              'NB',  46.565748,  -66.461914),
    ('Newfoundland and Labrador',  'NL',  53.135509,  -57.660435),
    ('Nova Scotia',                'NS',  44.681988,  -63.744311),
    ('Ontario',                    'ON',  51.253775,  -85.323214),
    ('Prince Edward Island',       'PE',  46.510712,  -63.416813),
    ('Quebec',                     'QC',  52.939916,  -73.549136),
    ('Saskatchewan',               'SK',  52.939916, -106.450860),
    ('Northwest Territories',      'NT',  64.825506, -124.845963),
    ('Nunavut',                    'NU',  70.453262,  -86.798981),
    ('Yukon',                      'YT',  64.282327, -135.000000)
) AS s(name, code, latitude, longitude)
ON CONFLICT DO NOTHING;


-- =========================================================================
-- Migration: 20250203165846_create_cities_table.up.sql
-- =========================================================================
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


-- =========================================================================
-- Migration: 20250203165847_create_addresses_table.up.sql
-- =========================================================================
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


-- =========================================================================
-- Migration: 20250203165848_create_domains_table.up.sql
-- =========================================================================
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


-- =========================================================================
-- Migration: 20251016175919_create_users_table.up.sql
-- =========================================================================
-- Up migration: create_users_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "users"
(
    id                   UUID         NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    first_name           VARCHAR      NOT NULL,
    last_name            VARCHAR      NOT NULL,
    email                VARCHAR      NOT NULL,
    phone                VARCHAR      NULL,
    password             VARCHAR      NOT NULL,
    password_reset_token VARCHAR      NULL,
    is_superuser         BOOLEAN      NOT NULL DEFAULT FALSE,
    is_internal          BOOLEAN      NOT NULL DEFAULT FALSE,
    verification_token   UUID         NULL DEFAULT (uuid_generate_v4()),
    last_logged_in_at    TIMESTAMP WITH TIME ZONE          DEFAULT NOW(),
    current_logged_in_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    date_of_birth        DATE                              NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

-- Create a unique constraint on name and deleted_at to ensure uniqueness
CREATE UNIQUE INDEX IF NOT EXISTS uq_user_email_deleted_at ON "users" (email) WHERE deleted_at IS NULL;

-- Create a composite index for name and deleted_at since we query them together
CREATE INDEX IF NOT EXISTS idx_user_email_deleted_at ON "users" (email, deleted_at);

-- System user:P@ssw0rd!
INSERT INTO "users" (first_name, last_name, email, phone, password, is_superuser, is_internal)
VALUES ('System', 'Admin', 'admin@imacals.com', NULL, '$argon2id$v=19$m=19456,t=2,p=1$Z3rdeGsJy39eJBb4Xdgg3Q$gTTSP0oqnSA3AEvjUC32PRfMSXouwxkp0bBZx1BE4qw', true, true);


-- =========================================================================
-- Migration: 20251016175920_create_organizations_table.up.sql
-- =========================================================================
-- Up migration: create_organizations_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "organizations"
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        VARCHAR NOT NULL,
    parent_id   UUID REFERENCES organizations(id) ON DELETE RESTRICT NULL,
    description VARCHAR,
    slug        VARCHAR UNIQUE NOT NULL,

    created_by  UUID REFERENCES users(id) ON DELETE RESTRICT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

-- Ensure parent lookup, hierarchy traversal, and child listing are fast
CREATE INDEX IF NOT EXISTS idx_org_parent
    ON organizations (parent_id)
    WHERE deleted_at IS NULL;

-- Fast soft-delete aware filtering
CREATE INDEX IF NOT EXISTS idx_org_deleted
    ON organizations (deleted_at);

-- Optimize name search, enforce active uniqueness for siblings if needed
CREATE INDEX IF NOT EXISTS idx_org_name
    ON organizations (name)
    WHERE deleted_at IS NULL;

-- Useful if querying by created_by (audit trails, ownership)
CREATE INDEX IF NOT EXISTS idx_org_created_by
    ON organizations (created_by)
    WHERE deleted_at IS NULL;

-- Fast slug lookup (slug is already UNIQUE, this optimizes queries)
CREATE INDEX IF NOT EXISTS idx_org_slug
    ON organizations (slug)
    WHERE deleted_at IS NULL;

-- Combined index for common queries involving name + parent (optional but recommended)
-- e.g., ensure no duplicate names under same parent
CREATE UNIQUE INDEX IF NOT EXISTS uq_org_parent_name
    ON organizations (parent_id, name)
    WHERE deleted_at IS NULL;

-- Seed initial organizations
WITH sys_user AS (
    SELECT id AS user_id FROM users WHERE email = 'admin@imacals.com' AND deleted_at IS NULL LIMIT 1
)
INSERT INTO organizations (name, slug, created_by, created_at)
SELECT o.name, o.slug, su.user_id,
    CASE
--      Make imacals the first organization created this help in ordering
        WHEN o.slug = 'imacals'
            THEN NOW() - INTERVAL '1 hour'
    ELSE NOW()
END AS created_at
FROM sys_user su
CROSS JOIN (VALUES ('Imacals', 'imacals')) AS o(name, slug)
ON CONFLICT (slug) DO NOTHING;




-- =========================================================================
-- Migration: 20251021133610_create_roles_table.up.sql
-- =========================================================================
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Roles table
CREATE TABLE IF NOT EXISTS roles
(
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            VARCHAR NOT NULL,
    title           VARCHAR NOT NULL,
    description     VARCHAR NOT NULL,
    organization_id UUID REFERENCES organizations(id) ON DELETE RESTRICT NULL,  -- NULL = system/global role

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

-- Index: ensure role names are unique PER tenant, ignoring soft deletes
-- Global roles (organization_id NULL) also enforced
CREATE UNIQUE INDEX IF NOT EXISTS uq_roles_tenant_name_active
    ON roles (organization_id, name)
    WHERE deleted_at IS NULL;

-- Lookup index for tenant-scoped queries
CREATE INDEX IF NOT EXISTS idx_roles_tenant
    ON roles (organization_id)
    WHERE deleted_at IS NULL;

-- Lookup index for name searches (with soft deletes)
CREATE INDEX IF NOT EXISTS idx_roles_name_deleted
    ON roles (name, deleted_at);


INSERT INTO "roles" ("name", "title", "description", "organization_id")
VALUES
    ('admin', 'Admin', 'Admin role', NULL),
    ('ai', 'Ai', 'Ai role', NULL),
    ('broker', 'Broker', 'Broker role', NULL),
    ('contractor', 'Contractor', 'Contractor role', NULL),
    ('hml', 'Hml', 'Hml role', NULL),
    ('insurance', 'Insurance', 'Insurance role', NULL),
    ('operator', 'Operator', 'Operator role', NULL),
    ('project-manager', 'Project-manager', 'Project-manager role', NULL),
    ('realtor', 'Realtor', 'Realtor role', NULL),
    ('super-admin', 'Super-admin', 'Super-admin role', NULL);

-- =========================================================================
-- Migration: 20251021140516_create_permissions_table.up.sql
-- =========================================================================
-- Up migration: create_permissions_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "permissions"
(
    id   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR UNIQUE NOT NULL
);

INSERT INTO "permissions" ("name") VALUES
    ('properties.view'),
    ('properties.create'),
    ('properties.update'),
    ('properties.delete'),
    ('properties.sync-images'),
    ('properties.view-comps'),
    ('properties.save-comp'),
    ('properties.view-as-lead'),

    ('renovations.view'),
    ('renovations.view-own'),
    ('renovations.create'),
    ('renovations.update'),
    ('renovations.delete'),
    ('renovations.view-dates'),
    ('renovations.view-phases'),
    ('renovations.view-budget'),
    ('renovations.manage-budget'),
    ('renovations.view-invoices'),
    ('renovations.view-own-invoices'),
    ('renovations.create-invoice'),
    ('renovations.approve-invoice'),
    ('renovations.pay-invoice'),
    ('renovations.view-invoice-history'),
    ('renovations.accept-offer'),
    ('renovations.decline-offer'),
    ('renovations.view-comps'),

    ('calendars.view'),
    ('calendars.create'),
    ('calendars.update'),
    ('calendars.delete'),
    ('calendars.update-scope-of-work'),
    ('calendars.view-labels'),

    ('scopes-of-work.view'),
    ('scopes-of-work.view-phases'),
    ('scopes-of-work.create'),
    ('scopes-of-work.update'),
    ('scopes-of-work.delete'),
    ('scopes-of-work.view-modifications'),

    ('tasks.view'),
    ('tasks.view-own'),
    ('tasks.create'),
    ('tasks.update'),
    ('tasks.complete'),
    ('tasks.delete'),
    ('tasks.undo'),
    ('tasks.upload-pictures'),

    ('contractors.view'),
    ('contractors.create'),
    ('contractors.update'),
    ('contractors.delete'),
    ('contractors.invite'),
    ('contractors.view-invites'),
    ('contractors.create-invite'),
    ('contractors.accept-invite'),
    ('contractors.decline-invite'),

    ('invoices.view'),
    ('invoices.view-own'),
    ('invoices.create'),
    ('invoices.update'),
    ('invoices.approve'),
    ('invoices.reject'),
    ('invoices.pay'),
    ('invoices.view-history'),
    ('invoices.view-grouped'),

    ('permits.view'),
    ('permits.create'),
    ('permits.update'),
    ('permits.delete'),
    ('permits.upload'),

    ('change-orders.view'),
    ('change-orders.create'),
    ('change-orders.update'),
    ('change-orders.approve'),
    ('change-orders.delete'),
    ('change-orders.view-phased'),

    ('budget-items.view'),
    ('budget-items.create'),
    ('budget-items.update'),
    ('budget-items.delete'),
    ('budget-items.find'),

    ('room-photos.view'),
    ('room-photos.upload'),
    ('room-photos.delete'),

    ('leads.view'),
    ('leads.create'),
    ('leads.update'),
    ('leads.delete'),
    ('leads.discard'),
    ('leads.view-status'),

    ('offers.view'),
    ('offers.create'),
    ('offers.update'),
    ('offers.delete'),
    ('offers.view-status'),
    ('offers.view-modifications'),

    ('bids.view'),
    ('bids.create'),
    ('bids.update'),
    ('bids.delete'),
    ('bids.assign'),

    ('contracts.view'),
    ('contracts.view-all'),
    ('contracts.create'),
    ('contracts.update'),
    ('contracts.delete'),
    ('contracts.view-templates'),
    ('contracts.create-template'),
    ('contracts.update-template'),
    ('contracts.delete-template'),

    ('maps.view'),
    ('maps.create'),
    ('maps.update'),
    ('maps.delete'),
    ('maps.view-untagged-properties'),

    ('polygons.create'),
    ('polygons.update'),
    ('polygons.delete'),
    ('polygons.add-neighbor'),
    ('polygons.delete-neighbor'),
    ('polygons.update-zone'),
    ('polygons.delete-zone'),

    ('polygon-zones.view'),
    ('polygon-zones.create'),
    ('polygon-zones.update'),
    ('polygon-zones.delete'),

    ('marketing.view'),
    ('marketing.generate-content'),
    ('marketing.preview'),
    ('marketing.send-email'),
    ('marketing.view-queue'),
    ('marketing.manage-queue'),

    ('files.view'),
    ('files.upload'),
    ('files.download'),

    ('users.view'),
    ('users.create'),
    ('users.update'),
    ('users.delete'),
    ('users.manage-roles'),
    ('users.manage-permissions'),

    ('roles.view'),
    ('roles.create'),
    ('roles.update'),
    ('roles.delete'),

    ('banks.verification-required'),
    ('banks.view'),
    ('banks.view-own'),
    ('banks.create'),
    ('banks.update'),
    ('banks.delete'),
    ('banks.verify'),
    ('banks.create-customer'),
    ('banks.generate-session'),
    ('banks.view-status'),

    ('listings.view'),
    ('listings.create'),
    ('listings.update'),
    ('listings.delete'),

    ('notes.view'),
    ('notes.create'),
    ('notes.update'),
    ('notes.delete'),

    ('flows.view'),
    ('flows.create'),
    ('flows.update'),
    ('flows.delete'),

    ('datatables.view'),
    ('datatables.create'),
    ('datatables.update'),
    ('datatables.delete'),

    ('preferences.view'),
    ('preferences.update'),

    ('feedback.view'),
    ('feedback.create'),
    ('feedback.update'),
    ('feedback.delete'),

    ('domains.view'),
    ('domains.create'),
    ('domains.update'),
    ('domains.delete'),

    ('dashboard.view'),
    ('dashboard.view-analytics'),

    ('webhooks.manage'),
    ('ai.use'),
    ('request-logs.view'),

    ('config.view'),
    ('config.update'),

    ('calendar-titles.view'),
    ('calendar-titles.create'),
    ('calendar-titles.update'),
    ('calendar-titles.delete'),

    ('financial-documents.view'),
    ('financial-documents.create'),
    ('financial-documents.update'),
    ('financial-documents.delete'),
    ('financial-documents.delete-files'),

    ('email-tracking.view'),

    ('micro-flows.view'),
    ('micro-flows.create'),
    ('micro-flows.update'),
    ('micro-flows.delete'),

    ('groups.view'),
    ('groups.create'),
    ('groups.update'),
    ('groups.delete'),

    ('pdf.generate'),
    ('static-maps.generate'),

    ('integrations.view'),
    ('integrations.create'),
    ('integrations.update'),
    ('integrations.delete'),

    ('attributes.view'),
    ('attributes.create'),
    ('attributes.update'),
    ('attributes.delete');


-- =========================================================================
-- Migration: 20251021141345_create_role_permissions_table.up.sql
-- =========================================================================
-- Up migration: create_role_permissions_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "role_permissions"
(
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- Reverse lookup: "which roles have this permission?" — the compound PK only
-- supports role_id-first scans, so permission_id needs its own index.
CREATE INDEX IF NOT EXISTS idx_role_permissions_permission_id
    ON role_permissions (permission_id);

-- Admin gets all permissions.
INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT r.id, p.id
FROM "roles" r
CROSS JOIN "permissions" p
WHERE r.name = 'admin' AND r.deleted_at IS NULL
ON CONFLICT DO NOTHING;

-- Other role permissions from the PHP seeder.
INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT r.id, p.id
FROM (
    VALUES
     -- Operator
     ('operator', 'properties.view'),
     ('operator', 'properties.update'),
     ('operator', 'properties.view-comps'),
     ('operator', 'properties.save-comp'),
     ('operator', 'properties.view-as-lead'),

     ('operator', 'renovations.view'),
     ('operator', 'renovations.create'),
     ('operator', 'renovations.view-dates'),
     ('operator', 'renovations.view-phases'),
     ('operator', 'renovations.view-budget'),
     ('operator', 'renovations.manage-budget'),
     ('operator', 'renovations.view-invoices'),
     ('operator', 'renovations.approve-invoice'),
     ('operator', 'renovations.pay-invoice'),
     ('operator', 'renovations.view-invoice-history'),
     ('operator', 'renovations.accept-offer'),
     ('operator', 'renovations.decline-offer'),
     ('operator', 'renovations.view-comps'),

     ('operator', 'calendars.view'),
     ('operator', 'calendars.update'),
     ('operator', 'calendars.update-scope-of-work'),
     ('operator', 'calendars.view-labels'),

     ('operator', 'scopes-of-work.view'),
     ('operator', 'scopes-of-work.view-phases'),
     ('operator', 'scopes-of-work.update'),

     ('operator', 'tasks.view'),
     ('operator', 'tasks.complete'),

     ('operator', 'contractors.view'),
     ('operator', 'contractors.invite'),
     ('operator', 'contractors.view-invites'),
     ('operator', 'contractors.create-invite'),

     ('operator', 'invoices.view'),
     ('operator', 'invoices.approve'),
     ('operator', 'invoices.reject'),
     ('operator', 'invoices.pay'),
     ('operator', 'invoices.view-history'),
     ('operator', 'invoices.view-grouped'),

     ('operator', 'permits.view'),
     ('operator', 'permits.update'),

     ('operator', 'change-orders.view'),
     ('operator', 'change-orders.create'),
     ('operator', 'change-orders.update'),
     ('operator', 'change-orders.approve'),
     ('operator', 'change-orders.view-phased'),

     ('operator', 'budget-items.view'),
     ('operator', 'budget-items.create'),
     ('operator', 'budget-items.update'),
     ('operator', 'budget-items.delete'),
     ('operator', 'budget-items.find'),

     ('operator', 'banks.verification-required'),
     ('operator', 'banks.view'),
     ('operator', 'banks.view-own'),
     ('operator', 'banks.create'),
     ('operator', 'banks.update'),
     ('operator', 'banks.delete'),
     ('operator', 'banks.verify'),
     ('operator', 'banks.create-customer'),
     ('operator', 'banks.generate-session'),
     ('operator', 'banks.view-status'),

     ('operator', 'files.view'),
     ('operator', 'files.upload'),
     ('operator', 'files.download'),

     ('operator', 'feedback.view'),
     ('operator', 'feedback.create'),

     ('operator', 'calendar-titles.view'),

     ('operator', 'financial-documents.view'),
     ('operator', 'financial-documents.create'),
     ('operator', 'financial-documents.update'),
     ('operator', 'financial-documents.delete'),
     ('operator', 'financial-documents.delete-files'),

     ('operator', 'contracts.view'),
     ('operator', 'contracts.view-all'),

     ('operator', 'dashboard.view'),

     -- Project Manager
     ('project-manager', 'renovations.view'),
     ('project-manager', 'renovations.view-dates'),
     ('project-manager', 'renovations.view-phases'),
     ('project-manager', 'renovations.view-budget'),
     ('project-manager', 'renovations.view-invoices'),
     ('project-manager', 'renovations.view-invoice-history'),
     ('project-manager', 'renovations.view-comps'),

     ('project-manager', 'calendars.view'),
     ('project-manager', 'calendars.view-labels'),
     ('project-manager', 'calendars.update-scope-of-work'),

     ('project-manager', 'scopes-of-work.view'),
     ('project-manager', 'scopes-of-work.view-phases'),

     ('project-manager', 'tasks.view'),
     ('project-manager', 'tasks.update'),

     ('project-manager', 'room-photos.view'),
     ('project-manager', 'room-photos.upload'),
     ('project-manager', 'room-photos.delete'),

     ('project-manager', 'contractors.view'),
     ('project-manager', 'contractors.view-invites'),

     ('project-manager', 'invoices.view'),
     ('project-manager', 'invoices.approve'),
     ('project-manager', 'invoices.reject'),
     ('project-manager', 'invoices.view-history'),
     ('project-manager', 'invoices.view-grouped'),

     ('project-manager', 'permits.view'),
     ('project-manager', 'permits.update'),

     ('project-manager', 'change-orders.view'),
     ('project-manager', 'change-orders.view-phased'),
     ('project-manager', 'change-orders.create'),
     ('project-manager', 'change-orders.update'),

     ('project-manager', 'budget-items.view'),
     ('project-manager', 'budget-items.create'),
     ('project-manager', 'budget-items.update'),
     ('project-manager', 'budget-items.delete'),
     ('project-manager', 'budget-items.find'),

     ('project-manager', 'calendar-titles.view'),

     ('project-manager', 'files.view'),
     ('project-manager', 'files.download'),

     ('project-manager', 'feedback.view'),
     ('project-manager', 'feedback.create'),

     ('project-manager', 'contracts.view'),

     ('project-manager', 'dashboard.view'),

     -- Contractor
     ('contractor', 'renovations.view'),
     ('contractor', 'renovations.view-own'),
     ('contractor', 'renovations.view-own-invoices'),
     ('contractor', 'renovations.create-invoice'),

     ('contractor', 'calendars.view'),
     ('contractor', 'calendars.view-labels'),

     ('contractor', 'scopes-of-work.view'),
     ('contractor', 'scopes-of-work.view-phases'),

     ('contractor', 'tasks.view-own'),
     ('contractor', 'tasks.update'),
     ('contractor', 'tasks.complete'),
     ('contractor', 'tasks.upload-pictures'),

     ('contractor', 'contractors.view'),
     ('contractor', 'contractors.view-invites'),
     ('contractor', 'contractors.accept-invite'),
     ('contractor', 'contractors.decline-invite'),

     ('contractor', 'invoices.view-own'),
     ('contractor', 'invoices.create'),
     ('contractor', 'invoices.update'),
     ('contractor', 'invoices.view-history'),
     ('contractor', 'invoices.view-grouped'),

     ('contractor', 'permits.view'),
     ('contractor', 'permits.upload'),

     ('contractor', 'room-photos.view'),
     ('contractor', 'room-photos.upload'),
     ('contractor', 'room-photos.delete'),

     ('contractor', 'banks.verification-required'),
     ('contractor', 'banks.view-own'),
     ('contractor', 'banks.create'),
     ('contractor', 'banks.update'),
     ('contractor', 'banks.delete'),
     ('contractor', 'banks.verify'),
     ('contractor', 'banks.create-customer'),
     ('contractor', 'banks.generate-session'),
     ('contractor', 'banks.view-status'),

     ('contractor', 'files.view'),
     ('contractor', 'files.upload'),
     ('contractor', 'files.download'),

     ('contractor', 'feedback.create'),

     ('contractor', 'contracts.view'),

     ('contractor', 'calendar-titles.view')
) AS rp(role_name, permission_name)
 JOIN "roles" r  ON r.name = rp.role_name  AND r.deleted_at IS NULL
 JOIN "permissions" p ON p.name = rp.permission_name
ON CONFLICT DO NOTHING;


-- =========================================================================
-- Migration: 20260127182935_create_organization_users_table.up.sql
-- =========================================================================
-- Up migration: create_organization_users_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "organization_users"
(
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    added_by        UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    role_id         UUID REFERENCES roles(id) NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ NULL
);

-- Prevent duplicate user ↔ organization relationships
CREATE UNIQUE INDEX IF NOT EXISTS idx_organization_users_unique_user_org
    ON organization_users (user_id, organization_id)
    WHERE deleted_at IS NULL;

-- Fast lookup of all users in an organization
CREATE INDEX IF NOT EXISTS idx_organization_users_organization_id
    ON organization_users (organization_id)
    WHERE deleted_at IS NULL;

-- Fast lookup of all organizations for a user
CREATE INDEX IF NOT EXISTS idx_organization_users_user_id
    ON organization_users (user_id)
    WHERE deleted_at IS NULL;

-- Optional: audit / admin queries (who added whom)
CREATE INDEX IF NOT EXISTS idx_organization_users_added_by
    ON organization_users (added_by);

INSERT INTO organization_users (user_id, organization_id, added_by)
SELECT u.id, o.id, u.id
FROM users u
JOIN organizations o ON o.slug = 'imacals'
WHERE u.email = 'admin@imacals.com' AND u.deleted_at IS NULL AND o.deleted_at IS NULL
ON CONFLICT (user_id, organization_id) WHERE deleted_at IS NULL DO NOTHING;


-- =========================================================================
-- Migration: 20260130174045_create_organization_users_permissions_table.up.sql
-- =========================================================================
-- Up migration: create_organization_users_permissions_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "organization_users_permissions"
(
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_users_id UUID NOT NULL REFERENCES organization_users(id) ON DELETE RESTRICT,
    permission_id         UUID NOT NULL REFERENCES permissions(id) ON DELETE RESTRICT,

    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_organization_users_permissions_id_active
    ON organization_users_permissions (id)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX idx_org_user_permissions_unique_active
    ON organization_users_permissions (organization_users_id, permission_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_org_user_permissions_org_user
    ON organization_users_permissions (organization_users_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_org_user_permissions_permission
    ON organization_users_permissions (permission_id)
    WHERE deleted_at IS NULL;

-- Cascade soft-delete from organization_users to organization_users_permissions
CREATE TRIGGER trg_soft_delete_org_user_permissions_on_org_user_delete
    AFTER UPDATE OF deleted_at ON organization_users
    FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_fk('organization_users_permissions', 'organization_users_id');



-- =========================================================================
-- Migration: 20260510222203_create_polygons_table.up.sql
-- =========================================================================
CREATE TABLE IF NOT EXISTS "polygons"
(
    id          UUID        PRIMARY KEY DEFAULT uuid_generate_v4(),
    created_by  UUID        NOT NULL REFERENCES users(id),
    -- GeoJSON-style array of {lat, lng} objects matching Google Maps format.
    coordinates JSONB       NOT NULL,
    city_id     UUID        REFERENCES cities(id),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS polygons_city_id_index
    ON polygons (city_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS polygons_created_by_index
    ON polygons (created_by)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering on listings.
CREATE INDEX IF NOT EXISTS polygons_deleted_at_index
    ON polygons (deleted_at);

-- Bidirectional neighbor relationships between polygons (used for comps proximity tiers).
CREATE TABLE IF NOT EXISTS "polygon_neighbors"
(
    polygon_id          UUID NOT NULL REFERENCES polygons(id) ON DELETE CASCADE,
    neighbor_polygon_id UUID NOT NULL REFERENCES polygons(id) ON DELETE CASCADE,
    PRIMARY KEY (polygon_id, neighbor_polygon_id),
    CHECK (polygon_id != neighbor_polygon_id)
);

CREATE INDEX IF NOT EXISTS polygon_neighbors_polygon_id_index ON polygon_neighbors (polygon_id);

-- Reverse-direction lookup: "who treats me as a neighbor?". The compound PK
-- only supports polygon_id-first scans.
CREATE INDEX IF NOT EXISTS polygon_neighbors_neighbor_polygon_id_index
    ON polygon_neighbors (neighbor_polygon_id);


-- =========================================================================
-- Migration: 20260510222220_create_organization_user_role_table.up.sql
-- =========================================================================
-- Job-title / profession concept, completely separate from permission-granting roles.
-- Roles (admin, super-admin) grant what a user CAN DO.
-- Organization user roles (contractor, broker, …) describe WHAT a user IS.

CREATE TABLE IF NOT EXISTS "organization_user_role" (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            VARCHAR NOT NULL,
    title           VARCHAR NOT NULL,
    description     VARCHAR NOT NULL DEFAULT '',
    organization_id UUID REFERENCES organizations(id) ON DELETE RESTRICT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ NULL
);

-- Name unique per tenant; global entries (NULL org) share the same uniqueness domain.
CREATE UNIQUE INDEX IF NOT EXISTS uq_org_user_role_name_active
    ON organization_user_role (COALESCE(organization_id, '00000000-0000-0000-0000-000000000000'::UUID), name)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_org_user_role_tenant
    ON organization_user_role (organization_id)
    WHERE deleted_at IS NULL;

-- Seed all profession entries that are currently in the roles table.
INSERT INTO organization_user_role (name, title, description, organization_id)
SELECT name, title, description, organization_id
FROM roles
WHERE name NOT IN ('admin', 'super-admin') AND deleted_at IS NULL;

-- Permission bundles per job title, mirroring role_permissions for the migrated entries.
CREATE TABLE IF NOT EXISTS "organization_user_role_permissions" (
    user_role_id  UUID NOT NULL REFERENCES organization_user_role(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (user_role_id, permission_id)
);

CREATE INDEX IF NOT EXISTS idx_org_user_role_permissions_user_role
    ON organization_user_role_permissions (user_role_id);

-- Reverse lookup: "which job-titles grant this permission?" — the compound PK
-- only supports user_role_id-first scans.
CREATE INDEX IF NOT EXISTS idx_org_user_role_permissions_permission
    ON organization_user_role_permissions (permission_id);

-- Copy existing permission assignments across.
INSERT INTO organization_user_role_permissions (user_role_id, permission_id)
SELECT ur.id, rp.permission_id
FROM organization_user_role ur
JOIN roles r ON r.name = ur.name AND r.deleted_at IS NULL
JOIN role_permissions rp ON rp.role_id = r.id;

-- Link each org membership to a job title.
ALTER TABLE organization_users ADD COLUMN user_role_id UUID REFERENCES organization_user_role(id) NULL;

-- Remove profession entries from the roles table; CASCADE cleans role_permissions.
DELETE FROM roles WHERE name NOT IN ('admin', 'super-admin');


-- =========================================================================
-- Migration: 20260511100000_create_zones_table.up.sql
-- =========================================================================
CREATE TABLE IF NOT EXISTS "polygon_zones" (
    id         UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    name       VARCHAR NOT NULL,
    color      VARCHAR   NOT NULL DEFAULT '#6366F1',
    created_by UUID         NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Audit / "zones I created" listings.
CREATE INDEX IF NOT EXISTS polygon_zones_created_by_index
    ON polygon_zones (created_by)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering on listings.
CREATE INDEX IF NOT EXISTS polygon_zones_deleted_at_index
    ON polygon_zones (deleted_at);

ALTER TABLE polygons ADD COLUMN IF NOT EXISTS polygon_zone_id UUID REFERENCES polygon_zones(id) NULL;

-- "List polygons in zone X" is the canonical query for the map sidebar.
CREATE INDEX IF NOT EXISTS polygons_polygon_zone_id_index
    ON polygons (polygon_zone_id)
    WHERE deleted_at IS NULL;


-- =========================================================================
-- Migration: 20260511110001_create_user_documents_table.up.sql
-- =========================================================================
-- Superseded: document/file storage is handled by the polymorphic `files` table.
-- This migration is intentionally a no-op.


-- =========================================================================
-- Migration: 20260511110002_create_user_bank_accounts_table.up.sql
-- =========================================================================
CREATE TABLE IF NOT EXISTS user_bank_accounts (
    id                   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id              UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    bank_name            TEXT NOT NULL,
    account_holder_name  TEXT NOT NULL,
    account_type         VARCHAR NOT NULL DEFAULT 'checking',
    account_number       TEXT NOT NULL,
    routing_number       TEXT NOT NULL,
    is_primary           BOOLEAN NOT NULL DEFAULT FALSE,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at           TIMESTAMPTZ
);

CREATE INDEX idx_user_bank_accounts_user_id ON user_bank_accounts (user_id) WHERE deleted_at IS NULL;

-- Enforce: at most one active primary account per user.
CREATE UNIQUE INDEX uq_user_bank_accounts_user_primary
    ON user_bank_accounts (user_id)
    WHERE is_primary = TRUE AND deleted_at IS NULL;


-- =========================================================================
-- Migration: 20260511260000_create_domain_system_users_table.up.sql
-- =========================================================================
-- domain_system_users: maps a platform user to a role (broker/realtor) within a domain.
-- Only one active assignment per (domain_id, role) is allowed at any time.
CREATE TABLE IF NOT EXISTS "domain_system_users" (
    id         UUID        NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    domain_id  UUID        NOT NULL REFERENCES domains(id),
    user_id    UUID        NOT NULL REFERENCES users(id),
    -- 'broker' or 'realtor' — the system role this user fills for this domain
    role       VARCHAR     NOT NULL,
    created_by UUID        NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

-- Enforce one active broker and one active realtor per domain.
CREATE UNIQUE INDEX uq_domain_system_users_domain_role
    ON domain_system_users (domain_id, role) WHERE deleted_at IS NULL;


-- =========================================================================
-- Migration: 20260511290000_add_system_user_eligible_to_user_roles.up.sql
-- =========================================================================
-- Marks which job-title roles may be used as system users on domain_system_users.
-- Only hml, insurance, broker, and realtor are eligible.
ALTER TABLE organization_user_role
    ADD COLUMN system_user_eligible BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE organization_user_role
SET    system_user_eligible = TRUE
WHERE  name IN ('hml', 'insurance', 'broker', 'realtor');


-- =========================================================================
-- Migration: 20260511300000_refactor_domain_system_users_role_fk.up.sql
-- =========================================================================
-- Replace the free-text role column with a proper FK to organization_user_role.
-- Drop and recreate so existing dev data (if any) is cleared cleanly.
DROP TABLE IF EXISTS domain_system_users;

CREATE TABLE IF NOT EXISTS "domain_system_users" (
    id           UUID        NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    domain_id    UUID        NOT NULL REFERENCES domains(id),
    user_id      UUID        NOT NULL REFERENCES users(id),
    user_role_id UUID        NOT NULL REFERENCES organization_user_role(id),
    created_by   UUID        NOT NULL REFERENCES users(id),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at   TIMESTAMPTZ NULL
);

-- One active assignment per (domain, job-title role).
CREATE UNIQUE INDEX uq_domain_system_users_domain_role
    ON domain_system_users (domain_id, user_role_id) WHERE deleted_at IS NULL;

-- "Which system roles does this user hold?" — common when loading a profile.
CREATE INDEX idx_domain_system_users_user_id
    ON domain_system_users (user_id) WHERE deleted_at IS NULL;

-- "Which users fill this job-title role across domains?"
CREATE INDEX idx_domain_system_users_user_role_id
    ON domain_system_users (user_role_id) WHERE deleted_at IS NULL;

-- Audit lookup: who created this assignment.
CREATE INDEX idx_domain_system_users_created_by
    ON domain_system_users (created_by);


-- =========================================================================
-- Migration: 20260511320000_create_files_table.up.sql
-- =========================================================================
CREATE TABLE IF NOT EXISTS files (
    id             UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    created_by     UUID         NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    fileable_type  VARCHAR NOT NULL,
    fileable_id    UUID         NOT NULL,
    type           VARCHAR NOT NULL,
    name           VARCHAR NOT NULL,
    absolute_path  TEXT         NOT NULL,
    relative_path  TEXT         NOT NULL,
    size           BIGINT       NOT NULL,
    mime_type      VARCHAR NOT NULL,
    created_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at     TIMESTAMPTZ
);

-- =========================
-- Indexes
-- =========================

-- Primary lookup: all files for a given owner row.
CREATE INDEX IF NOT EXISTS files_fileable_index
    ON files (fileable_type, fileable_id)
    WHERE deleted_at IS NULL;

-- Type-scoped lookup: e.g. fetch only the signature for a user.
CREATE INDEX IF NOT EXISTS files_fileable_type_index
    ON files (fileable_type, fileable_id, type)
    WHERE deleted_at IS NULL;

-- FK: uploader.
CREATE INDEX IF NOT EXISTS files_created_by_index
    ON files (created_by)
    WHERE deleted_at IS NULL;

-- Soft-delete filtering for audit / restore queries.
CREATE INDEX IF NOT EXISTS files_deleted_at_index
    ON files (deleted_at);


-- =========================================================================
-- Migration: 20260511350000_create_integrations_table.up.sql
-- =========================================================================
-- Third-party system connections scoped to an org and domain.
-- organization_id defaults to the "imacals" platform org when not supplied by the caller.
CREATE TABLE IF NOT EXISTS integrations (
    id               UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id  UUID         NOT NULL REFERENCES organizations(id),
    domain_id        UUID         NOT NULL REFERENCES domains(id),
    created_by       UUID         NOT NULL REFERENCES users(id),
    name             VARCHAR      NOT NULL,
    slug             VARCHAR      NOT NULL,
    integration_type VARCHAR(50)  NOT NULL DEFAULT 'custom',
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at       TIMESTAMPTZ
);

-- =========================
-- Indexes
-- =========================

-- FK lookups for joins.
CREATE INDEX IF NOT EXISTS integrations_organization_id_index
    ON integrations (organization_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS integrations_domain_id_index
    ON integrations (domain_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS integrations_created_by_index
    ON integrations (created_by)
    WHERE deleted_at IS NULL;

-- Slug lookup within an org.
CREATE INDEX IF NOT EXISTS integrations_slug_index
    ON integrations (slug)
    WHERE deleted_at IS NULL;

-- Type lookup for filtering by integration kind.
CREATE INDEX IF NOT EXISTS integrations_type_index
    ON integrations (integration_type)
    WHERE deleted_at IS NULL;

-- Name must be unique per (org, domain) but reusable after soft-delete.
CREATE UNIQUE INDEX IF NOT EXISTS uq_integrations_name_org_domain_active
    ON integrations (name, organization_id, domain_id)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering.
CREATE INDEX IF NOT EXISTS integrations_deleted_at_index
    ON integrations (deleted_at);


-- =========================================================================
-- Migration: 20260511360000_create_attributes_table.up.sql
-- =========================================================================
-- Key/value config entries attached polymorphically to any owner entity.
-- Primary use: storing credentials/settings for integrations (e.g. RETS server auth params).
-- Encrypted values are stored raw in `value`; is_encrypted signals the service to decrypt.
CREATE TABLE IF NOT EXISTS attributes (
    id                 UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    created_by         UUID         NOT NULL REFERENCES users(id),
    attributeable_type VARCHAR NOT NULL,
    attributeable_id   UUID         NOT NULL,
    name               VARCHAR NOT NULL,
    value              TEXT,
    type               VARCHAR NOT NULL,
    is_encrypted       BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at         TIMESTAMPTZ
);

-- =========================
-- Indexes
-- =========================

-- Primary lookup: all attributes for a given owner row.
CREATE INDEX IF NOT EXISTS attributes_attributeable_index
    ON attributes (attributeable_type, attributeable_id)
    WHERE deleted_at IS NULL;

-- FK: who created the attribute entry.
CREATE INDEX IF NOT EXISTS attributes_created_by_index
    ON attributes (created_by)
    WHERE deleted_at IS NULL;

-- Attribute name must be unique per owner (e.g. only one "url" per integration).
CREATE UNIQUE INDEX IF NOT EXISTS uq_attributes_owner_name_active
    ON attributes (attributeable_type, attributeable_id, name)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering.
CREATE INDEX IF NOT EXISTS attributes_deleted_at_index
    ON attributes (deleted_at);

-- =========================
-- Cascade trigger helper
-- =========================

-- Separate from soft_delete_cascade_by_owner() which targets owner_type/owner_id columns.
-- This variant targets the attributeable_type/attributeable_id naming convention used here.
CREATE OR REPLACE FUNCTION soft_delete_cascade_by_attributeable()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        UPDATE attributes
        SET deleted_at = NEW.deleted_at
        WHERE attributeable_type = TG_TABLE_NAME
          AND attributeable_id   = NEW.id
          AND deleted_at IS NULL;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Attributes vanish from listings when their parent integration is soft-deleted.
CREATE TRIGGER trg_soft_delete_attributes_on_integration_delete
    AFTER UPDATE OF deleted_at ON integrations
    FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_attributeable();


-- =========================================================================
-- Migration: 20260804120000_add_category_and_enabled_to_integrations.up.sql
-- =========================================================================
-- Provider family + live flag for integrations.
--
-- `integration_category` groups providers that are interchangeable (any Email provider can send a
-- campaign), and `is_enabled` marks which one of a family is actually live. Together they let the
-- resolver answer "who sends mail right now?" from the database on every send — which is what
-- makes credentials editable from the dashboard without an app restart. Env vars only ever seed
-- these rows; they are never read again after the seed.
ALTER TABLE integrations
    ADD COLUMN IF NOT EXISTS integration_category VARCHAR(50) NOT NULL DEFAULT 'other',
    ADD COLUMN IF NOT EXISTS is_enabled           BOOLEAN     NOT NULL DEFAULT TRUE;

-- Derive the category for rows that predate the column. Always kept in step with
-- integration_type by the service layer, so the two columns can never disagree.
UPDATE integrations
SET integration_category = CASE
        WHEN integration_type IN ('smtp', 'log', 'mailgun', 'mailchimp', 'google', 'outlook')
            THEN 'email'
        WHEN integration_type = 'zero-bounce'
            THEN 'email-validation'
        ELSE 'other'
    END;

-- imacals is an email-campaign platform: the MLS provider types it inherited (rets/reso) are gone
-- from IntegrationType, so a surviving row of that type would fail to deserialize on read.
-- Soft-delete rather than drop, so the rows stay auditable.
UPDATE integrations
SET deleted_at = NOW()
WHERE integration_type IN ('rets', 'reso')
  AND deleted_at IS NULL;

-- =========================
-- Indexes
-- =========================

-- The resolver's hot path: the enabled provider for a family.
CREATE INDEX IF NOT EXISTS integrations_category_index
    ON integrations (integration_category)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS integrations_category_enabled_index
    ON integrations (integration_category, is_enabled)
    WHERE deleted_at IS NULL;

-- At most one live provider per family, per scope — enforced here rather than in application
-- logic, so two concurrent "make this one active" requests can't both win. 'other' (the Custom
-- catch-all) is exempt: those rows are free-form config, not interchangeable providers.
CREATE UNIQUE INDEX IF NOT EXISTS uq_integrations_enabled_per_category
    ON integrations (organization_id, domain_id, integration_category)
    WHERE is_enabled = TRUE
      AND deleted_at IS NULL
      AND integration_category <> 'other';


-- =========================================================================
-- Migration: 20260821100000_create_categories_table.up.sql
-- =========================================================================
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS categories (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    domain_id   UUID NOT NULL REFERENCES domains(id),
    created_by  UUID REFERENCES users(id),
    name        VARCHAR NOT NULL,
    slug        VARCHAR NOT NULL,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

-- =========================
-- Indexes
-- =========================

-- FK lookups for joins.
CREATE INDEX IF NOT EXISTS idx_categories_domain_id
    ON categories (domain_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_categories_created_by
    ON categories (created_by)
    WHERE deleted_at IS NULL;

-- Slug must be unique within a domain but reusable after soft-delete.
CREATE UNIQUE INDEX IF NOT EXISTS uq_categories_domain_slug_active
    ON categories (domain_id, slug)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering on listings / audit queries.
CREATE INDEX IF NOT EXISTS idx_categories_deleted_at
    ON categories (deleted_at);

-- Seed default product categories under the default domain.
INSERT INTO categories (domain_id, name, slug, description)
SELECT d.id, c.name, c.slug, c.description
FROM domains d
CROSS JOIN (
    VALUES
        ('Foodstuff', 'foodstuff', 'Grains, oils, flour, seasonings and wholesale staple foods'),
        ('Household', 'household', 'Detergents, soaps, cleaning supplies and home maintenance'),
        ('Beverages', 'beverages', 'Water, soft drinks, juices and malt beverages')
) AS c(name, slug, description)
WHERE d.slug = 'default-us' OR d.slug = 'default-ng'
LIMIT 3
ON CONFLICT DO NOTHING;


-- =========================================================================
-- Migration: 20260821100001_create_products_table.up.sql
-- =========================================================================
CREATE TABLE IF NOT EXISTS products (
    id                 UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id    UUID NOT NULL REFERENCES organizations(id),
    domain_id          UUID NOT NULL REFERENCES domains(id),
    category_id        UUID NOT NULL REFERENCES categories(id),
    created_by         UUID NOT NULL REFERENCES users(id),
    name               VARCHAR NOT NULL,
    slug               VARCHAR NOT NULL,
    description        TEXT,
    unit               VARCHAR NOT NULL,
    unit_price_kobo    BIGINT NOT NULL,
    min_order_quantity INTEGER NOT NULL DEFAULT 1,
    in_stock           BOOLEAN NOT NULL DEFAULT TRUE,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at         TIMESTAMPTZ
);

-- =========================
-- Indexes
-- =========================

-- FK lookups for joins.
CREATE INDEX IF NOT EXISTS products_organization_id_index
    ON products (organization_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS products_domain_id_index
    ON products (domain_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS products_category_id_index
    ON products (category_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS products_created_by_index
    ON products (created_by)
    WHERE deleted_at IS NULL;

-- Slug must be unique per tenant but reusable after soft-delete.
CREATE UNIQUE INDEX IF NOT EXISTS uq_products_org_slug_active
    ON products (organization_id, slug)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering on listings / audit queries.
CREATE INDEX IF NOT EXISTS products_deleted_at_index
    ON products (deleted_at);

-- =========================
-- Triggers
-- =========================

-- Cascade soft-delete from category to product.
CREATE TRIGGER trg_soft_delete_products_on_category_delete
    AFTER UPDATE OF deleted_at ON categories
    FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_fk('products', 'category_id');


-- =========================================================================
-- Migration: 20260821100002_seed_product_permissions.up.sql
-- =========================================================================
-- Up migration: seed_product_permissions
INSERT INTO permissions (name) VALUES
    ('products.view'),
    ('products.create'),
    ('products.update'),
    ('products.delete'),
    ('categories.view'),
    ('categories.create'),
    ('categories.update'),
    ('categories.delete')
ON CONFLICT (name) DO NOTHING;

-- Grant product and category permissions to the admin role
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
  AND (p.name LIKE 'products.%' OR p.name LIKE 'categories.%')
ON CONFLICT DO NOTHING;


