-- Replace the free-text role column with a proper FK to organization_user_role.
-- Drop and recreate so existing dev data (if any) is cleared cleanly.
DROP TABLE IF EXISTS domain_system_users;

CREATE TABLE IF NOT EXISTS "domain_system_users" (
    id           UUID        NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    domain_id    UUID        NOT NULL REFERENCES domains(id),
    user_id      UUID        NOT NULL REFERENCES users(id),
    user_role_id UUID        NOT NULL REFERENCES organization_user_role(id),
    created_by   UUID        NOT NULL REFERENCES users(id),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at   TIMESTAMPTZ NULL
);

-- One active assignment per (domain, job-title role).
CREATE UNIQUE INDEX uq_domain_system_users_domain_role
    ON domain_system_users (domain_id, user_role_id) WHERE deleted_at IS NULL;

-- "Which system roles does this user hold?" — common when loading a profile.
CREATE INDEX idx_domain_system_users_user_id
    ON domain_system_users (user_id) WHERE deleted_at IS NULL;

-- "Which users fill this job-title role across domains?"
CREATE INDEX idx_domain_system_users_user_role_id
    ON domain_system_users (user_role_id) WHERE deleted_at IS NULL;

-- Audit lookup: who created this assignment.
CREATE INDEX idx_domain_system_users_created_by
    ON domain_system_users (created_by);
