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
