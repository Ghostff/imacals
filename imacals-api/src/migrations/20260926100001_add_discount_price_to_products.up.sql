-- Add discount price in integer kobo to products table.
ALTER TABLE products
    ADD COLUMN IF NOT EXISTS discount_price_kobo BIGINT NULL;

-- Enforce that promotional discount price cannot be zero/negative and must be strictly less than the regular unit price.
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'chk_products_discount_less_than_unit_price'
    ) THEN
        ALTER TABLE products
            ADD CONSTRAINT chk_products_discount_less_than_unit_price
            CHECK (discount_price_kobo IS NULL OR (discount_price_kobo > 0 AND discount_price_kobo < unit_price_kobo));
    END IF;
END $$;

-- =========================
-- Indexes
-- =========================

-- Soft-delete aware index for queries targeting discounted products (promotions, deals, clearance).
CREATE INDEX IF NOT EXISTS products_discount_price_kobo_index
    ON products (discount_price_kobo)
    WHERE deleted_at IS NULL AND discount_price_kobo IS NOT NULL;
