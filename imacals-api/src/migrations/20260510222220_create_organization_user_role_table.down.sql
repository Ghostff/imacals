-- Restore profession entries into roles.
INSERT INTO roles (name, title, description, organization_id)
SELECT name, title, description, organization_id
FROM organization_user_role
WHERE deleted_at IS NULL
ON CONFLICT DO NOTHING;

-- Restore role_permissions.
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, urp.permission_id
FROM organization_user_role_permissions urp
JOIN organization_user_role ur ON ur.id = urp.user_role_id
JOIN roles r ON r.name = ur.name AND r.deleted_at IS NULL
ON CONFLICT DO NOTHING;

ALTER TABLE organization_users DROP COLUMN IF EXISTS user_role_id;
DROP TABLE IF EXISTS organization_user_role_permissions;
DROP TABLE IF EXISTS organization_user_role;
