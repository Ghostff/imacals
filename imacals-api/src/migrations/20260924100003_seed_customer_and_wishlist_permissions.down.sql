-- Down migration: seed_customer_and_wishlist_permissions
DELETE FROM role_permissions
WHERE permission_id IN (SELECT id FROM permissions WHERE name LIKE 'customers.%' OR name LIKE 'wishlists.%');

DELETE FROM permissions WHERE name LIKE 'customers.%' OR name LIKE 'wishlists.%';
