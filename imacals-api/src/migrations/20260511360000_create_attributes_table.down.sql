DROP TRIGGER IF EXISTS trg_soft_delete_attributes_on_integration_delete ON integrations;
DROP FUNCTION IF EXISTS soft_delete_cascade_by_attributeable();
DROP TABLE IF EXISTS attributes;
