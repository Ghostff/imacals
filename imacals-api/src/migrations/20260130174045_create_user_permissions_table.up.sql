-- Permissions granted directly to a user. Replaces the old organization-scoped grant table:
-- Imacals is one business, so there is no tenant to scope a grant by.
--
-- Roles are a bundle, not a live link — assigning a role resolves its permissions and writes them
-- here. Changing a role later does not retro-fit existing users; permissions must be re-synced.
-- That keeps a deliberate per-user grant from being silently wiped by a role edit.
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "user_permissions"
(
    id            UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id       UUID NOT NULL REFERENCES users(id)       ON DELETE RESTRICT,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE RESTRICT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

-- =========================
-- Indexes
-- =========================

-- A user holds a given permission at most once, but the pair is reusable after a soft-delete.
CREATE UNIQUE INDEX IF NOT EXISTS uq_user_permissions_active
    ON user_permissions (user_id, permission_id)
    WHERE deleted_at IS NULL;

-- The hot path: every gate! call loads one user's permissions.
CREATE INDEX IF NOT EXISTS idx_user_permissions_user
    ON user_permissions (user_id)
    WHERE deleted_at IS NULL;

-- Reverse lookup: "who holds this permission?"
CREATE INDEX IF NOT EXISTS idx_user_permissions_permission
    ON user_permissions (permission_id)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering for audit/restore.
CREATE INDEX IF NOT EXISTS idx_user_permissions_deleted_at
    ON user_permissions (deleted_at);

-- A user's grants vanish from listings when the user is soft-deleted.
CREATE TRIGGER trg_soft_delete_user_permissions_on_user_delete
    AFTER UPDATE OF deleted_at ON users
    FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_fk('user_permissions', 'user_id');

-- Sync the seeded system admin's direct grants from the admin role, so permission checks work on a
-- fresh install even for a non-superuser. Runs here rather than in the role_permissions migration
-- because this table does not exist until now.
INSERT INTO "user_permissions" ("user_id", "permission_id")
SELECT u.id, rp.permission_id
FROM "users" u
JOIN "role_permissions" rp ON rp.role_id = u.role_id
WHERE u.email = 'admin@imacals.com' AND u.deleted_at IS NULL
ON CONFLICT DO NOTHING;
