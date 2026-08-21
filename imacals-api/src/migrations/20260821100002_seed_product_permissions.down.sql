-- Down migration: rollback product permissions
DELETE FROM role_permissions
WHERE permission_id IN (
    SELECT id FROM permissions
    WHERE name LIKE 'products.%' OR name LIKE 'categories.%'
);

DELETE FROM permissions
WHERE name LIKE 'products.%' OR name LIKE 'categories.%';
