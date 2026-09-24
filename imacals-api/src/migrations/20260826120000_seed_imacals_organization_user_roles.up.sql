-- Seed the Imacals ecommerce and distribution job titles / professions into organization_user_role.
-- Global roles (organization_id IS NULL) are available to all organizations.

INSERT INTO organization_user_role (name, title, description, organization_id, system_user_eligible)
VALUES
    ('order-desk',       'Order Desk',          'Takes phone orders and enters them on customer behalf',   NULL, FALSE),
    ('warehouse',        'Warehouse Picker',    'Picks and packs orders in Aba warehouse; adjusts stock',  NULL, FALSE),
    ('dispatch',         'Dispatch Manager',    'Assigns orders to vehicles and routes; confirms delivery', NULL, FALSE),
    ('rider',            'Rider / Driver',      'Carries the load and captures proof of delivery',         NULL, FALSE),
    ('accounts',         'Accounts / Finance',  'Reconciles payments, issues refunds',                    NULL, FALSE),
    ('sales-rep',        'Sales Representative','Handles client relationships and bulk wholesale orders',  NULL, FALSE),
    ('store-manager',    'Store Manager',       'Oversees warehouse and store distribution operations',    NULL, FALSE),
    ('customer-support', 'Customer Support',    'Assists customers with enquiries and order assistance',   NULL, FALSE),
    ('customer',         'Customer',            'Online and retail customer account',                     NULL, FALSE)
ON CONFLICT (COALESCE(organization_id, '00000000-0000-0000-0000-000000000000'::UUID), name)
WHERE deleted_at IS NULL
DO NOTHING;
