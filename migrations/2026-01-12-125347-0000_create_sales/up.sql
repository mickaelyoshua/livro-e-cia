CREATE TABLE sales (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    seller_id UUID NOT NULL REFERENCES employees(id) ON DELETE RESTRICT,
    subtotal DECIMAL(10,2) NOT NULL CHECK (subtotal >= 0),
    discount DECIMAL(10,2) NOT NULL DEFAULT 0 CHECK (discount >= 0),
    total DECIMAL(10,2) NOT NULL CHECK (total >= 0),
    payment_method payment_method NOT NULL,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_sales_seller_id ON sales(seller_id);
CREATE INDEX idx_sales_created_at ON sales(created_at DESC);

CREATE TRIGGER update_sales_updated_at
	BEFORE UPDATE ON sales
	FOR EACH ROW
	EXECUTE FUNCTION update_updated_at_column();
