-- Up migration: create_customers_table
-- A buyer. May exist without a users row — phone orders create one from a name and number.

CREATE TABLE IF NOT EXISTS customers (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id),
    full_name       VARCHAR NOT NULL,
    phone           VARCHAR,
    email           VARCHAR,
    -- Linked user account: NULL for phone-only customers, populated for online customers.
    user_id         UUID REFERENCES users(id),
    created_by      UUID REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

-- =========================
-- Indexes
-- =========================

-- FK lookups for joins.
CREATE INDEX IF NOT EXISTS customers_organization_id_index
    ON customers (organization_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS customers_user_id_index
    ON customers (user_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS customers_created_by_index
    ON customers (created_by)
    WHERE deleted_at IS NULL;

-- Phone is how staff find an existing phone customer when taking a call.
CREATE INDEX IF NOT EXISTS customers_phone_index
    ON customers (organization_id, phone)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering.
CREATE INDEX IF NOT EXISTS customers_deleted_at_index
    ON customers (deleted_at);
