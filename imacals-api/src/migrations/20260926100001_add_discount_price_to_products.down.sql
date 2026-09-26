DROP INDEX IF EXISTS products_discount_price_kobo_index;

ALTER TABLE products
    DROP CONSTRAINT IF EXISTS chk_products_discount_less_than_unit_price;

ALTER TABLE products
    DROP COLUMN IF EXISTS discount_price_kobo;
