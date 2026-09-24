-- Up migration: create_wishlist_items_table
-- A single product saved on a wishlist. A product can appear at most once per active wishlist.

CREATE TABLE IF NOT EXISTS wishlist_items (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    wishlist_id UUID NOT NULL REFERENCES wishlists(id),
    product_id  UUID NOT NULL REFERENCES products(id),
    notes       TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

-- =========================
-- Indexes
-- =========================

-- FK lookups for joins.
CREATE INDEX IF NOT EXISTS wishlist_items_wishlist_id_index
    ON wishlist_items (wishlist_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS wishlist_items_product_id_index
    ON wishlist_items (product_id)
    WHERE deleted_at IS NULL;

-- A product appears at most once per active wishlist.
CREATE UNIQUE INDEX IF NOT EXISTS uq_wishlist_items_wishlist_product_active
    ON wishlist_items (wishlist_id, product_id)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering.
CREATE INDEX IF NOT EXISTS wishlist_items_deleted_at_index
    ON wishlist_items (deleted_at);

-- =========================
-- Triggers
-- =========================

-- Soft-deleting a wishlist removes its items; soft-deleting a product removes it from every wishlist.
CREATE TRIGGER trg_soft_delete_wishlist_items_on_wishlist_delete
    AFTER UPDATE OF deleted_at ON wishlists
    FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_fk('wishlist_items', 'wishlist_id');

CREATE TRIGGER trg_soft_delete_wishlist_items_on_product_delete
    AFTER UPDATE OF deleted_at ON products
    FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_fk('wishlist_items', 'product_id');
