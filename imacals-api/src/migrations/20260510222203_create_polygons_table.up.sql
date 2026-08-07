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
