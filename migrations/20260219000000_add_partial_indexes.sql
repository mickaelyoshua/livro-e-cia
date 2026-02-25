-- Partial indexes (filtered)
CREATE INDEX idx_employees_active ON employees(id) WHERE is_active = TRUE;
CREATE INDEX idx_products_active ON products(id) WHERE is_active = TRUE;
CREATE INDEX idx_refresh_token_active ON refresh_token_families(id) WHERE is_revoked = FALSE;

-- Foreign key indexes (PostgreSQL does not auto-index FK columns)
CREATE INDEX idx_sales_seller_id ON sales(seller_id);
CREATE INDEX idx_products_category_id ON products(category_id);
CREATE INDEX idx_refresh_token_employee_id ON refresh_token_families(employee_id);
