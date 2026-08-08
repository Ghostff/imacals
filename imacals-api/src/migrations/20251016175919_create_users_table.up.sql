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
    -- The role whose permission bundle this user was last synced from. Nullable: a superuser needs
    -- no role, and the seeded system admin is created before any role row exists.
    role_id              UUID         NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

-- Create a unique constraint on name and deleted_at to ensure uniqueness
CREATE UNIQUE INDEX IF NOT EXISTS uq_user_email_deleted_at ON "users" (email) WHERE deleted_at IS NULL;

-- Create a composite index for name and deleted_at since we query them together
CREATE INDEX IF NOT EXISTS idx_user_email_deleted_at ON "users" (email, deleted_at);

-- FK lookup: "who holds this role?". The constraint itself is added in the roles migration, which
-- runs after this one — a table cannot reference one that does not exist yet.
CREATE INDEX IF NOT EXISTS idx_users_role_id ON "users" (role_id) WHERE deleted_at IS NULL;

-- System user:P@ssw0rd!
INSERT INTO "users" (first_name, last_name, email, phone, password, is_superuser, is_internal)
VALUES ('System', 'Admin', 'admin@imacals.com', NULL, '$argon2id$v=19$m=19456,t=2,p=1$Z3rdeGsJy39eJBb4Xdgg3Q$gTTSP0oqnSA3AEvjUC32PRfMSXouwxkp0bBZx1BE4qw', true, true);
