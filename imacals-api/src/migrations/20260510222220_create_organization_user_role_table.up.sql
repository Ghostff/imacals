-- Job-title / profession concept, completely separate from permission-granting roles.
-- Roles (admin, super-admin) grant what a user CAN DO.
-- Organization user roles (contractor, broker, …) describe WHAT a user IS.

CREATE TABLE IF NOT EXISTS "organization_user_role" (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            VARCHAR NOT NULL,
    title           VARCHAR NOT NULL,
    description     VARCHAR NOT NULL DEFAULT '',
    organization_id UUID REFERENCES organizations(id) ON DELETE RESTRICT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ NULL
);

-- Name unique per tenant; global entries (NULL org) share the same uniqueness domain.
CREATE UNIQUE INDEX IF NOT EXISTS uq_org_user_role_name_active
    ON organization_user_role (COALESCE(organization_id, '00000000-0000-0000-0000-000000000000'::UUID), name)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_org_user_role_tenant
    ON organization_user_role (organization_id)
    WHERE deleted_at IS NULL;

-- Seed all profession entries that are currently in the roles table.
INSERT INTO organization_user_role (name, title, description, organization_id)
SELECT name, title, description, organization_id
FROM roles
WHERE name NOT IN ('admin', 'super-admin') AND deleted_at IS NULL;

-- Permission bundles per job title, mirroring role_permissions for the migrated entries.
CREATE TABLE IF NOT EXISTS "organization_user_role_permissions" (
    user_role_id  UUID NOT NULL REFERENCES organization_user_role(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (user_role_id, permission_id)
);

CREATE INDEX IF NOT EXISTS idx_org_user_role_permissions_user_role
    ON organization_user_role_permissions (user_role_id);

-- Reverse lookup: "which job-titles grant this permission?" — the compound PK
-- only supports user_role_id-first scans.
CREATE INDEX IF NOT EXISTS idx_org_user_role_permissions_permission
    ON organization_user_role_permissions (permission_id);

-- Copy existing permission assignments across.
INSERT INTO organization_user_role_permissions (user_role_id, permission_id)
SELECT ur.id, rp.permission_id
FROM organization_user_role ur
JOIN roles r ON r.name = ur.name AND r.deleted_at IS NULL
JOIN role_permissions rp ON rp.role_id = r.id;

-- Link each org membership to a job title.
ALTER TABLE organization_users ADD COLUMN user_role_id UUID REFERENCES organization_user_role(id) NULL;

-- Remove profession entries from the roles table; CASCADE cleans role_permissions.
DELETE FROM roles WHERE name NOT IN ('admin', 'super-admin');
