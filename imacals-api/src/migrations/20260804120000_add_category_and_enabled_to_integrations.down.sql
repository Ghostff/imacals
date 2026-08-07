DROP INDEX IF EXISTS uq_integrations_enabled_per_category;
DROP INDEX IF EXISTS integrations_category_enabled_index;
DROP INDEX IF EXISTS integrations_category_index;

ALTER TABLE integrations
    DROP COLUMN IF EXISTS is_enabled,
    DROP COLUMN IF EXISTS integration_category;
