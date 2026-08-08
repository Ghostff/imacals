-- Up migration: create_permissions_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "permissions"
(
    id   UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR UNIQUE NOT NULL
);

-- Only permissions that something actually checks. A feature seeds its own permissions in its own
-- migration (see AGENTS.md) — do not add rows here for tables that do not exist yet, or the list
-- drifts back into fiction.
INSERT INTO "permissions" ("name") VALUES
    ('users.view'),
    ('users.create'),
    ('users.update'),
    ('users.delete'),
    ('users.manage-permissions'),

    ('roles.view'),
    ('roles.create'),
    ('roles.update'),
    ('roles.delete');
