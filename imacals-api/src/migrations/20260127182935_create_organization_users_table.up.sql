-- Up migration: create_organization_users_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "organization_users"
(
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    added_by        UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    role_id         UUID REFERENCES roles(id) NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ NULL
);

-- Prevent duplicate user ↔ organization relationships
CREATE UNIQUE INDEX IF NOT EXISTS idx_organization_users_unique_user_org
    ON organization_users (user_id, organization_id)
    WHERE deleted_at IS NULL;

-- Fast lookup of all users in an organization
CREATE INDEX IF NOT EXISTS idx_organization_users_organization_id
    ON organization_users (organization_id)
    WHERE deleted_at IS NULL;

-- Fast lookup of all organizations for a user
CREATE INDEX IF NOT EXISTS idx_organization_users_user_id
    ON organization_users (user_id)
    WHERE deleted_at IS NULL;

-- Optional: audit / admin queries (who added whom)
CREATE INDEX IF NOT EXISTS idx_organization_users_added_by
    ON organization_users (added_by);

INSERT INTO organization_users (user_id, organization_id, added_by)
SELECT u.id, o.id, u.id
FROM users u
JOIN organizations o ON o.slug = 'imacals'
WHERE u.email = 'admin@imacals.com' AND u.deleted_at IS NULL AND o.deleted_at IS NULL
ON CONFLICT (user_id, organization_id) WHERE deleted_at IS NULL DO NOTHING;
