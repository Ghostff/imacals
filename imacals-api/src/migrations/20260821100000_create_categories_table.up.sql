CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS categories (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    domain_id   UUID NOT NULL REFERENCES domains(id),
    created_by  UUID REFERENCES users(id),
    name        VARCHAR NOT NULL,
    slug        VARCHAR NOT NULL,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);

-- =========================
-- Indexes
-- =========================

-- FK lookups for joins.
CREATE INDEX IF NOT EXISTS idx_categories_domain_id
    ON categories (domain_id)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_categories_created_by
    ON categories (created_by)
    WHERE deleted_at IS NULL;

-- Slug must be unique within a domain but reusable after soft-delete.
CREATE UNIQUE INDEX IF NOT EXISTS uq_categories_domain_slug_active
    ON categories (domain_id, slug)
    WHERE deleted_at IS NULL;

-- Soft-delete aware filtering on listings / audit queries.
CREATE INDEX IF NOT EXISTS idx_categories_deleted_at
    ON categories (deleted_at);

-- Seed default product categories under the default domain.
INSERT INTO categories (domain_id, name, slug, description)
SELECT d.id, c.name, c.slug, c.description
FROM domains d
CROSS JOIN (
    VALUES
        ('Foodstuff', 'foodstuff', 'Grains, oils, flour, seasonings and wholesale staple foods'),
        ('Household', 'household', 'Detergents, soaps, cleaning supplies and home maintenance'),
        ('Beverages', 'beverages', 'Water, soft drinks, juices and malt beverages')
) AS c(name, slug, description)
WHERE d.slug = 'default-us' OR d.slug = 'default-ng'
LIMIT 3
ON CONFLICT DO NOTHING;
