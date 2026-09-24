-- Down migration: create_wishlist_items_table
DROP TRIGGER IF EXISTS trg_soft_delete_wishlist_items_on_wishlist_delete ON wishlists;
DROP TRIGGER IF EXISTS trg_soft_delete_wishlist_items_on_product_delete ON products;
DROP TABLE IF EXISTS wishlist_items;
