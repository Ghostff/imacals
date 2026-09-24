-- Revert seeded Imacals organization user roles.
DELETE FROM organization_user_role
WHERE organization_id IS NULL
  AND name IN (
    'order-desk',
    'warehouse',
    'dispatch',
    'rider',
    'accounts',
    'sales-rep',
    'store-manager',
    'customer-support',
    'customer'
  );
