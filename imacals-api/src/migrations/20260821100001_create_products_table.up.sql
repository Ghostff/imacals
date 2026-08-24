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
