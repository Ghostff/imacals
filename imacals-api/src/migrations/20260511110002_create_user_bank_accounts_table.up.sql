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
