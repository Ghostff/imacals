DROP INDEX IF EXISTS products_is_tax_exempt_index;

ALTER TABLE products
    DROP COLUMN IF EXISTS tax_rate_basis_points,
    DROP COLUMN IF EXISTS is_tax_exempt;
