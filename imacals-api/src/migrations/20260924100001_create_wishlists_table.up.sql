-- Up migration: create_wishlists_table
-- A saved list of products a customer wants. Owned by a customer so both online customers
-- (linked via user_id) and phone customers (no user row) can have one.

CREATE TABLE IF NOT EXISTS wishlists (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    customer_id     UUID NOT NULL REFERENCES customers(id),
    name            VARCHAR NOT NULL,
    description     TEXT,
    -- The user who created the row — a logged-in customer or a staff member on the phone.
    created_by      UUID REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

-- =========================
-- Indexes
-- =========================

-- FK lookups for joins.
CREATE INDEX IF NOT EXISTS wishlists_organization_id_index
    ON wishlists (organization_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS wishlists_customer_id_index
    ON wishlists (customer_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS wishlists_created_by_index
    ON wishlists (created_by)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering.
CREATE INDEX IF NOT EXISTS wishlists_deleted_at_index
    ON wishlists (deleted_at);

-- =========================
-- Triggers
-- =========================

-- Soft-deleting a customer removes their wishlists so the listing stays coherent.
CREATE TRIGGER trg_soft_delete_wishlists_on_customer_delete
    AFTER UPDATE OF deleted_at ON customers
    FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_fk('wishlists', 'customer_id');
