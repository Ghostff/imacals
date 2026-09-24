-- Down migration: create_wishlists_table
DROP TRIGGER IF EXISTS trg_soft_delete_wishlists_on_customer_delete ON customers;
DROP TABLE IF EXISTS wishlists;
