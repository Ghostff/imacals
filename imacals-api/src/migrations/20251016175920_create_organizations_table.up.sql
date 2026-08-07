-- Up migration: create_organizations_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "organizations"
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        VARCHAR NOT NULL,
    parent_id   UUID REFERENCES organizations(id) ON DELETE RESTRICT NULL,
    description VARCHAR,
    slug        VARCHAR UNIQUE NOT NULL,

    created_by  UUID REFERENCES users(id) ON DELETE RESTRICT NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

-- Ensure parent lookup, hierarchy traversal, and child listing are fast
CREATE INDEX IF NOT EXISTS idx_org_parent
    ON organizations (parent_id)
    WHERE deleted_at IS NULL;

-- Fast soft-delete aware filtering
CREATE INDEX IF NOT EXISTS idx_org_deleted
    ON organizations (deleted_at);

-- Optimize name search, enforce active uniqueness for siblings if needed
CREATE INDEX IF NOT EXISTS idx_org_name
    ON organizations (name)
    WHERE deleted_at IS NULL;

-- Useful if querying by created_by (audit trails, ownership)
CREATE INDEX IF NOT EXISTS idx_org_created_by
    ON organizations (created_by)
    WHERE deleted_at IS NULL;

-- Fast slug lookup (slug is already UNIQUE, this optimizes queries)
CREATE INDEX IF NOT EXISTS idx_org_slug
    ON organizations (slug)
    WHERE deleted_at IS NULL;

-- Combined index for common queries involving name + parent (optional but recommended)
-- e.g., ensure no duplicate names under same parent
CREATE UNIQUE INDEX IF NOT EXISTS uq_org_parent_name
    ON organizations (parent_id, name)
    WHERE deleted_at IS NULL;

-- Seed initial organizations
WITH sys_user AS (
    SELECT id AS user_id FROM users WHERE email = 'admin@imacals.com' AND deleted_at IS NULL LIMIT 1
)
INSERT INTO organizations (name, slug, created_by, created_at)
SELECT o.name, o.slug, su.user_id,
    CASE
--      Make imacals the first organization created this help in ordering
        WHEN o.slug = 'imacals'
            THEN NOW() - INTERVAL '1 hour'
    ELSE NOW()
END AS created_at
FROM sys_user su
CROSS JOIN (VALUES ('Imacals', 'imacals')) AS o(name, slug)
ON CONFLICT (slug) DO NOTHING;


