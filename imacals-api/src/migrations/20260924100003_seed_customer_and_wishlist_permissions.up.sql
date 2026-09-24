-- Up migration: seed_customer_and_wishlist_permissions
INSERT INTO permissions (name) VALUES
    ('customers.view'),
    ('customers.create'),
    ('customers.update'),
    ('customers.delete'),
    ('wishlists.view'),
    ('wishlists.create'),
    ('wishlists.update'),
    ('wishlists.delete')
ON CONFLICT (name) DO NOTHING;

-- Grant customer and wishlist permissions to the admin role.
INSERT INTO role_permissions (role_id, permission_id)
SELECT r.id, p.id
FROM roles r
CROSS JOIN permissions p
WHERE r.name = 'admin'
  AND (p.name LIKE 'customers.%' OR p.name LIKE 'wishlists.%')
ON CONFLICT DO NOTHING;
