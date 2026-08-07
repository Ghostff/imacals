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
