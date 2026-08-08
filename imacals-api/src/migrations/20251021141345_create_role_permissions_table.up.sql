-- Up migration: create_role_permissions_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "role_permissions"
(
    role_id       UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- Reverse lookup: "which roles have this permission?" — the compound PK only
-- supports role_id-first scans, so permission_id needs its own index.
CREATE INDEX IF NOT EXISTS idx_role_permissions_permission_id
    ON role_permissions (permission_id);

-- Admin gets everything.
INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT r.id, p.id
FROM "roles" r
CROSS JOIN "permissions" p
WHERE r.name = 'admin' AND r.deleted_at IS NULL
ON CONFLICT DO NOTHING;

-- Everyone else gets what their job needs and nothing more. The warehouse role deliberately gets
-- nothing yet: picking and stock permissions arrive with the tables they act on.
INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT r.id, p.id
FROM (
    VALUES
     -- Order desk: needs to find a customer's record, never to change staff accounts.
     ('order-desk', 'users.view'),

     -- Dispatch: reads staff records when assigning a load to a rider.
     ('dispatch',   'users.view'),

     -- Accounts: reads staff records for reconciliation.
     ('accounts',   'users.view')
) AS rp(role_name, permission_name)
 JOIN "roles" r       ON r.name = rp.role_name AND r.deleted_at IS NULL
 JOIN "permissions" p ON p.name = rp.permission_name
ON CONFLICT DO NOTHING;
