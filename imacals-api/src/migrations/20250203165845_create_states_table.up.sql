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
