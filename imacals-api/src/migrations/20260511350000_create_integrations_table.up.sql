-- Third-party system connections scoped to an org and domain.
-- organization_id defaults to the "imacals" platform org when not supplied by the caller.
CREATE TABLE IF NOT EXISTS integrations (
    id               UUID         PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id  UUID         NOT NULL REFERENCES organizations(id),
    domain_id        UUID         NOT NULL REFERENCES domains(id),
    created_by       UUID         NOT NULL REFERENCES users(id),
    name             VARCHAR      NOT NULL,
    slug             VARCHAR      NOT NULL,
    integration_type VARCHAR(50)  NOT NULL DEFAULT 'custom',
    created_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at       TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    deleted_at       TIMESTAMPTZ
);

-- =========================
-- Indexes
-- =========================

-- FK lookups for joins.
CREATE INDEX IF NOT EXISTS integrations_organization_id_index
    ON integrations (organization_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS integrations_domain_id_index
    ON integrations (domain_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS integrations_created_by_index
    ON integrations (created_by)
    WHERE deleted_at IS NULL;

-- Slug lookup within an org.
CREATE INDEX IF NOT EXISTS integrations_slug_index
    ON integrations (slug)
    WHERE deleted_at IS NULL;

-- Type lookup for filtering by integration kind.
CREATE INDEX IF NOT EXISTS integrations_type_index
    ON integrations (integration_type)
    WHERE deleted_at IS NULL;

-- Name must be unique per (org, domain) but reusable after soft-delete.
CREATE UNIQUE INDEX IF NOT EXISTS uq_integrations_name_org_domain_active
    ON integrations (name, organization_id, domain_id)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering.
CREATE INDEX IF NOT EXISTS integrations_deleted_at_index
    ON integrations (deleted_at);
