-- domain_system_users: maps a platform user to a role (broker/realtor) within a domain.
-- Only one active assignment per (domain_id, role) is allowed at any time.
CREATE TABLE IF NOT EXISTS "domain_system_users" (
    id         UUID        NOT NULL PRIMARY KEY DEFAULT (uuid_generate_v4()),
    domain_id  UUID        NOT NULL REFERENCES domains(id),
    user_id    UUID        NOT NULL REFERENCES users(id),
    -- 'broker' or 'realtor' — the system role this user fills for this domain
    role       VARCHAR     NOT NULL,
    created_by UUID        NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

-- Enforce one active broker and one active realtor per domain.
CREATE UNIQUE INDEX uq_domain_system_users_domain_role
    ON domain_system_users (domain_id, role) WHERE deleted_at IS NULL;
