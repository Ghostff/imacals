CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- A role is a named bundle of permissions. Imacals is a single business, not a multi-tenant
-- platform, so roles are global — there is no tenant column to scope them by.
CREATE TABLE IF NOT EXISTS roles
(
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name        VARCHAR NOT NULL,
    title       VARCHAR NOT NULL,
    description VARCHAR NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ NULL
);

-- =========================
-- Indexes
-- =========================

-- Role names are unique, but a name becomes reusable once the old role is soft-deleted.
CREATE UNIQUE INDEX IF NOT EXISTS uq_roles_name_active
    ON roles (name)
    WHERE deleted_at IS NULL;

-- Lookup index for name searches that include soft-deleted rows (audit, restore).
CREATE INDEX IF NOT EXISTS idx_roles_name_deleted
    ON roles (name, deleted_at);

-- The roles an Imacals warehouse actually staffs. `super-admin` is never assigned to a person —
-- it is the bypass flag on users.is_superuser.
INSERT INTO "roles" ("name", "title", "description")
VALUES
    ('super-admin', 'Super Admin', 'Unrestricted system access'),
    ('admin',       'Admin',       'Full access: staff, catalogue, orders, settings'),
    ('order-desk',  'Order Desk',  'Takes phone orders and enters them for the customer'),
    ('warehouse',   'Warehouse',   'Picks and packs orders; adjusts stock'),
    ('dispatch',    'Dispatch',    'Assigns orders to vehicles and routes; confirms delivery'),
    ('accounts',    'Accounts',    'Reconciles payments and issues refunds');

-- users.role_id is declared in the users migration (which runs first) but can only be constrained
-- once roles exists. ON DELETE RESTRICT: a role with people on it must not vanish under them.
ALTER TABLE "users"
    ADD CONSTRAINT users_role_id_fkey
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE RESTRICT;

-- Put the seeded system admin on the admin role. Its permissions are granted in the
-- role_permissions migration, which runs after this one.
UPDATE "users"
SET    role_id = (SELECT id FROM roles WHERE name = 'admin' AND deleted_at IS NULL)
WHERE  email = 'admin@imacals.com';
