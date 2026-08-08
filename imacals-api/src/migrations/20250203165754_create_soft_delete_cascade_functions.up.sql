-- Up migration: create_soft_delete_cascade_functions
-- Reusable trigger functions for cascading soft-deletes.
-- Attach these to any table that has children using parent_id or owner_type/owner_id.

-- ─────────────────────────────────────────────────────────────────────────────
-- 1. parent_id cascade (self-referential tables, e.g. a category tree)
--    When a row's deleted_at transitions NULL → value, soft-delete all rows
--    in the SAME table whose parent_id = NEW.id.
-- ─────────────────────────────────────────────────────────────────────────────
CREATE OR REPLACE FUNCTION soft_delete_cascade_by_parent_id()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        EXECUTE format(
            'UPDATE %I SET deleted_at = $1 WHERE parent_id = $2 AND deleted_at IS NULL',
            TG_TABLE_NAME
        ) USING NEW.deleted_at, NEW.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ─────────────────────────────────────────────────────────────────────────────
-- 2. owner_type / owner_id cascade (polymorphic ownership)
--    When a row's deleted_at transitions NULL → value, soft-delete all rows
--    in the target table whose owner_type = TG_TABLE_NAME AND owner_id = NEW.id.
--    Pass the target table name via TG_ARGV[0].
--    Example attachment:
--      CREATE TRIGGER trg_soft_delete_notes_on_orders_delete
--          AFTER UPDATE OF deleted_at ON facility_units
--          FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_owner('notes');
-- ─────────────────────────────────────────────────────────────────────────────
CREATE OR REPLACE FUNCTION soft_delete_cascade_by_owner()
RETURNS TRIGGER AS $$
DECLARE
    target_table TEXT := TG_ARGV[0];
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        EXECUTE format(
            'UPDATE %I SET deleted_at = $1 WHERE owner_type = $2 AND owner_id = $3 AND deleted_at IS NULL',
            target_table
        ) USING NEW.deleted_at, TG_TABLE_NAME, NEW.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ─────────────────────────────────────────────────────────────────────────────
-- 3. Direct FK cascade (child table references parent via a named FK column)
--    When a row's deleted_at transitions NULL → value, soft-delete all rows
--    in the target table whose <fk_column> = NEW.id.
--    Pass target table name as TG_ARGV[0] and FK column name as TG_ARGV[1].
--    Example attachment:
--      CREATE TRIGGER trg_soft_delete_facility_units_on_facilities_delete
--          AFTER UPDATE OF deleted_at ON facilities
--          FOR EACH ROW EXECUTE FUNCTION soft_delete_cascade_by_fk('facility_units', 'facility_id');
-- ─────────────────────────────────────────────────────────────────────────────
CREATE OR REPLACE FUNCTION soft_delete_cascade_by_fk()
RETURNS TRIGGER AS $$
DECLARE
    target_table TEXT := TG_ARGV[0];
    fk_column    TEXT := TG_ARGV[1];
BEGIN
    IF NEW.deleted_at IS NOT NULL AND OLD.deleted_at IS NULL THEN
        EXECUTE format(
            'UPDATE %I SET deleted_at = $1 WHERE %I = $2 AND deleted_at IS NULL',
            target_table, fk_column
        ) USING NEW.deleted_at, NEW.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
