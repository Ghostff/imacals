-- Up migration: seed_product_permissions
INSERT INTO permissions (name) VALUES
    ('products.view'),
    ('products.create'),
    ('products.update'),
    ('products.delete'),
    ('categories.view'),
    ('categories.create'),
    ('categories.update'),
    ('categories.delete')
ON CONFLICT (name) DO NOTHING;

-- Grant product and category permissions to the admin role
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
  AND (p.name LIKE 'products.%' OR p.name LIKE 'categories.%')
ON CONFLICT DO NOTHING;
