-- Provider family + live flag for integrations.
--
-- `integration_category` groups providers that are interchangeable (any Email provider can send a
-- campaign), and `is_enabled` marks which one of a family is actually live. Together they let the
-- resolver answer "who sends mail right now?" from the database on every send — which is what
-- makes credentials editable from the dashboard without an app restart. Env vars only ever seed
-- these rows; they are never read again after the seed.
ALTER TABLE integrations
    ADD COLUMN IF NOT EXISTS integration_category VARCHAR(50) NOT NULL DEFAULT 'other',
    ADD COLUMN IF NOT EXISTS is_enabled           BOOLEAN     NOT NULL DEFAULT TRUE;

-- Derive the category for rows that predate the column. Always kept in step with
-- integration_type by the service layer, so the two columns can never disagree.
UPDATE integrations
SET integration_category = CASE
        WHEN integration_type IN ('smtp', 'log', 'mailgun', 'mailchimp', 'google', 'outlook')
            THEN 'email'
        WHEN integration_type = 'zero-bounce'
            THEN 'email-validation'
        ELSE 'other'
    END;

-- imacals is an email-campaign platform: the MLS provider types it inherited (rets/reso) are gone
-- from IntegrationType, so a surviving row of that type would fail to deserialize on read.
-- Soft-delete rather than drop, so the rows stay auditable.
UPDATE integrations
SET deleted_at = NOW()
WHERE integration_type IN ('rets', 'reso')
  AND deleted_at IS NULL;

-- =========================
-- Indexes
-- =========================

-- The resolver's hot path: the enabled provider for a family.
CREATE INDEX IF NOT EXISTS integrations_category_index
    ON integrations (integration_category)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS integrations_category_enabled_index
    ON integrations (integration_category, is_enabled)
    WHERE deleted_at IS NULL;

-- At most one live provider per family, per scope — enforced here rather than in application
-- logic, so two concurrent "make this one active" requests can't both win. 'other' (the Custom
-- catch-all) is exempt: those rows are free-form config, not interchangeable providers.
CREATE UNIQUE INDEX IF NOT EXISTS uq_integrations_enabled_per_category
    ON integrations (organization_id, domain_id, integration_category)
    WHERE is_enabled = TRUE
      AND deleted_at IS NULL
      AND integration_category <> 'other';
