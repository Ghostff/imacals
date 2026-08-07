CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Roles table
CREATE TABLE IF NOT EXISTS roles
(
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            VARCHAR NOT NULL,
    title           VARCHAR NOT NULL,
    description     VARCHAR NOT NULL,
    organization_id UUID REFERENCES organizations(id) ON DELETE RESTRICT NULL,  -- NULL = system/global role

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

-- Index: ensure role names are unique PER tenant, ignoring soft deletes
-- Global roles (organization_id NULL) also enforced
CREATE UNIQUE INDEX IF NOT EXISTS uq_roles_tenant_name_active
    ON roles (organization_id, name)
    WHERE deleted_at IS NULL;

-- Lookup index for tenant-scoped queries
CREATE INDEX IF NOT EXISTS idx_roles_tenant
    ON roles (organization_id)
    WHERE deleted_at IS NULL;

-- Lookup index for name searches (with soft deletes)
CREATE INDEX IF NOT EXISTS idx_roles_name_deleted
    ON roles (name, deleted_at);


INSERT INTO "roles" ("name", "title", "description", "organization_id")
VALUES
    ('admin', 'Admin', 'Admin role', NULL),
    ('ai', 'Ai', 'Ai role', NULL),
    ('broker', 'Broker', 'Broker role', NULL),
    ('contractor', 'Contractor', 'Contractor role', NULL),
    ('hml', 'Hml', 'Hml role', NULL),
    ('insurance', 'Insurance', 'Insurance role', NULL),
    ('operator', 'Operator', 'Operator role', NULL),
    ('project-manager', 'Project-manager', 'Project-manager role', NULL),
    ('realtor', 'Realtor', 'Realtor role', NULL),
    ('super-admin', 'Super-admin', 'Super-admin role', NULL);