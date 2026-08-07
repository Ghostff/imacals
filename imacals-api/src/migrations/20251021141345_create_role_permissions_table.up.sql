-- Up migration: create_role_permissions_table
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS "role_permissions"
(
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- Reverse lookup: "which roles have this permission?" — the compound PK only
-- supports role_id-first scans, so permission_id needs its own index.
CREATE INDEX IF NOT EXISTS idx_role_permissions_permission_id
    ON role_permissions (permission_id);

-- Admin gets all permissions.
INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT r.id, p.id
FROM "roles" r
CROSS JOIN "permissions" p
WHERE r.name = 'admin' AND r.deleted_at IS NULL
ON CONFLICT DO NOTHING;

-- Other role permissions from the PHP seeder.
INSERT INTO "role_permissions" ("role_id", "permission_id")
SELECT r.id, p.id
FROM (
    VALUES
     -- Operator
     ('operator', 'properties.view'),
     ('operator', 'properties.update'),
     ('operator', 'properties.view-comps'),
     ('operator', 'properties.save-comp'),
     ('operator', 'properties.view-as-lead'),

     ('operator', 'renovations.view'),
     ('operator', 'renovations.create'),
     ('operator', 'renovations.view-dates'),
     ('operator', 'renovations.view-phases'),
     ('operator', 'renovations.view-budget'),
     ('operator', 'renovations.manage-budget'),
     ('operator', 'renovations.view-invoices'),
     ('operator', 'renovations.approve-invoice'),
     ('operator', 'renovations.pay-invoice'),
     ('operator', 'renovations.view-invoice-history'),
     ('operator', 'renovations.accept-offer'),
     ('operator', 'renovations.decline-offer'),
     ('operator', 'renovations.view-comps'),

     ('operator', 'calendars.view'),
     ('operator', 'calendars.update'),
     ('operator', 'calendars.update-scope-of-work'),
     ('operator', 'calendars.view-labels'),

     ('operator', 'scopes-of-work.view'),
     ('operator', 'scopes-of-work.view-phases'),
     ('operator', 'scopes-of-work.update'),

     ('operator', 'tasks.view'),
     ('operator', 'tasks.complete'),

     ('operator', 'contractors.view'),
     ('operator', 'contractors.invite'),
     ('operator', 'contractors.view-invites'),
     ('operator', 'contractors.create-invite'),

     ('operator', 'invoices.view'),
     ('operator', 'invoices.approve'),
     ('operator', 'invoices.reject'),
     ('operator', 'invoices.pay'),
     ('operator', 'invoices.view-history'),
     ('operator', 'invoices.view-grouped'),

     ('operator', 'permits.view'),
     ('operator', 'permits.update'),

     ('operator', 'change-orders.view'),
     ('operator', 'change-orders.create'),
     ('operator', 'change-orders.update'),
     ('operator', 'change-orders.approve'),
     ('operator', 'change-orders.view-phased'),

     ('operator', 'budget-items.view'),
     ('operator', 'budget-items.create'),
     ('operator', 'budget-items.update'),
     ('operator', 'budget-items.delete'),
     ('operator', 'budget-items.find'),

     ('operator', 'banks.verification-required'),
     ('operator', 'banks.view'),
     ('operator', 'banks.view-own'),
     ('operator', 'banks.create'),
     ('operator', 'banks.update'),
     ('operator', 'banks.delete'),
     ('operator', 'banks.verify'),
     ('operator', 'banks.create-customer'),
     ('operator', 'banks.generate-session'),
     ('operator', 'banks.view-status'),

     ('operator', 'files.view'),
     ('operator', 'files.upload'),
     ('operator', 'files.download'),

     ('operator', 'feedback.view'),
     ('operator', 'feedback.create'),

     ('operator', 'calendar-titles.view'),

     ('operator', 'financial-documents.view'),
     ('operator', 'financial-documents.create'),
     ('operator', 'financial-documents.update'),
     ('operator', 'financial-documents.delete'),
     ('operator', 'financial-documents.delete-files'),

     ('operator', 'contracts.view'),
     ('operator', 'contracts.view-all'),

     ('operator', 'dashboard.view'),

     -- Project Manager
     ('project-manager', 'renovations.view'),
     ('project-manager', 'renovations.view-dates'),
     ('project-manager', 'renovations.view-phases'),
     ('project-manager', 'renovations.view-budget'),
     ('project-manager', 'renovations.view-invoices'),
     ('project-manager', 'renovations.view-invoice-history'),
     ('project-manager', 'renovations.view-comps'),

     ('project-manager', 'calendars.view'),
     ('project-manager', 'calendars.view-labels'),
     ('project-manager', 'calendars.update-scope-of-work'),

     ('project-manager', 'scopes-of-work.view'),
     ('project-manager', 'scopes-of-work.view-phases'),

     ('project-manager', 'tasks.view'),
     ('project-manager', 'tasks.update'),

     ('project-manager', 'room-photos.view'),
     ('project-manager', 'room-photos.upload'),
     ('project-manager', 'room-photos.delete'),

     ('project-manager', 'contractors.view'),
     ('project-manager', 'contractors.view-invites'),

     ('project-manager', 'invoices.view'),
     ('project-manager', 'invoices.approve'),
     ('project-manager', 'invoices.reject'),
     ('project-manager', 'invoices.view-history'),
     ('project-manager', 'invoices.view-grouped'),

     ('project-manager', 'permits.view'),
     ('project-manager', 'permits.update'),

     ('project-manager', 'change-orders.view'),
     ('project-manager', 'change-orders.view-phased'),
     ('project-manager', 'change-orders.create'),
     ('project-manager', 'change-orders.update'),

     ('project-manager', 'budget-items.view'),
     ('project-manager', 'budget-items.create'),
     ('project-manager', 'budget-items.update'),
     ('project-manager', 'budget-items.delete'),
     ('project-manager', 'budget-items.find'),

     ('project-manager', 'calendar-titles.view'),

     ('project-manager', 'files.view'),
     ('project-manager', 'files.download'),

     ('project-manager', 'feedback.view'),
     ('project-manager', 'feedback.create'),

     ('project-manager', 'contracts.view'),

     ('project-manager', 'dashboard.view'),

     -- Contractor
     ('contractor', 'renovations.view'),
     ('contractor', 'renovations.view-own'),
     ('contractor', 'renovations.view-own-invoices'),
     ('contractor', 'renovations.create-invoice'),

     ('contractor', 'calendars.view'),
     ('contractor', 'calendars.view-labels'),

     ('contractor', 'scopes-of-work.view'),
     ('contractor', 'scopes-of-work.view-phases'),

     ('contractor', 'tasks.view-own'),
     ('contractor', 'tasks.update'),
     ('contractor', 'tasks.complete'),
     ('contractor', 'tasks.upload-pictures'),

     ('contractor', 'contractors.view'),
     ('contractor', 'contractors.view-invites'),
     ('contractor', 'contractors.accept-invite'),
     ('contractor', 'contractors.decline-invite'),

     ('contractor', 'invoices.view-own'),
     ('contractor', 'invoices.create'),
     ('contractor', 'invoices.update'),
     ('contractor', 'invoices.view-history'),
     ('contractor', 'invoices.view-grouped'),

     ('contractor', 'permits.view'),
     ('contractor', 'permits.upload'),

     ('contractor', 'room-photos.view'),
     ('contractor', 'room-photos.upload'),
     ('contractor', 'room-photos.delete'),

     ('contractor', 'banks.verification-required'),
     ('contractor', 'banks.view-own'),
     ('contractor', 'banks.create'),
     ('contractor', 'banks.update'),
     ('contractor', 'banks.delete'),
     ('contractor', 'banks.verify'),
     ('contractor', 'banks.create-customer'),
     ('contractor', 'banks.generate-session'),
     ('contractor', 'banks.view-status'),

     ('contractor', 'files.view'),
     ('contractor', 'files.upload'),
     ('contractor', 'files.download'),

     ('contractor', 'feedback.create'),

     ('contractor', 'contracts.view'),

     ('contractor', 'calendar-titles.view')
) AS rp(role_name, permission_name)
 JOIN "roles" r  ON r.name = rp.role_name  AND r.deleted_at IS NULL
 JOIN "permissions" p ON p.name = rp.permission_name
ON CONFLICT DO NOTHING;
