DROP TRIGGER IF EXISTS update_products_updated_at ON products;
DROP INDEX IF EXISTS idx_products_category_id;
DROP INDEX IF EXISTS idx_products_title;
DROP INDEX IF EXISTS idx_products_author;
DROP INDEX IF EXISTS idx_products_is_active;
DROP INDEX IF EXISTS idx_products_category_created;
DROP TABLE IF EXISTS products;
