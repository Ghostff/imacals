-- Adds Nigerian VAT / tax configuration to products table.
-- Money is always integer kobo; tax rate is stored in basis points (750 basis points = 7.50% VAT).
-- Unprocessed agricultural produce / basic raw foodstuff can be marked as is_tax_exempt = TRUE.

ALTER TABLE products
    ADD COLUMN IF NOT EXISTS is_tax_exempt BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN IF NOT EXISTS tax_rate_basis_points INT NOT NULL DEFAULT 750;

-- Soft-delete aware index for tax reporting and catalogue tax filtering.
CREATE INDEX IF NOT EXISTS products_is_tax_exempt_index
    ON products (organization_id, is_tax_exempt)
    WHERE deleted_at IS NULL;
