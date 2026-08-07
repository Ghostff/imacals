DROP INDEX IF EXISTS uq_domain_system_users_domain_role;
ALTER TABLE domain_system_users DROP COLUMN IF EXISTS user_role_id;
ALTER TABLE domain_system_users ADD COLUMN role VARCHAR NOT NULL DEFAULT '';
CREATE UNIQUE INDEX uq_domain_system_users_domain_role
    ON domain_system_users (domain_id, role) WHERE deleted_at IS NULL;
