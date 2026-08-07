-- Up migration: create_organization_users_permissions_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "organization_users_permissions"
(
    id                    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_users_id UUID NOT NULL REFERENCES organization_users(id) ON DELETE RESTRICT,
    permission_id         UUID NOT NULL REFERENCES permissions(id) ON DELETE RESTRICT,

    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ NULL
);

CREATE INDEX IF NOT EXISTS idx_organization_users_permissions_id_active
    ON organization_users_permissions (id)
    WHERE deleted_at IS NULL;

CREATE UNIQUE INDEX idx_org_user_permissions_unique_active
    ON organization_users_permissions (organization_users_id, permission_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_org_user_permissions_org_user
    ON organization_users_permissions (organization_users_id)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_org_user_permissions_permission
    ON organization_users_permissions (permission_id)
    WHERE deleted_at IS NULL;

-- Cascade soft-delete from organization_users to organization_users_permissions
CREATE TRIGGER trg_soft_delete_org_user_permissions_on_org_user_delete
    AFTER UPDATE OF deleted_at ON organization_users
    FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_fk('organization_users_permissions', 'organization_users_id');

