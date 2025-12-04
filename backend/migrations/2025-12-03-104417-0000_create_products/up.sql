CREATE TABLE products (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	title VARCHAR(255) NOT NULL,
	author VARCHAR(255) NOT NULL,
	price DECIMAL(10,2) NOT NULL CHECK (price >= 0),
	stock_quantity INTEGER NOT NULL DEFAULT 0 CHECK (stock_quantity >= 0),
	publisher VARCHAR(255),
	publication_date DATE,
	category_id UUID NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
	description TEXT,
	cover_image_url VARCHAR(500),
	is_active BOOLEAN NOT NULL DEFAULT TRUE,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_products_category_id ON products(category_id);
CREATE INDEX idx_products_title ON products(title);
CREATE INDEX idx_products_author ON products(author);
CREATE INDEX idx_products_is_active ON products(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_products_category_created ON products(category_id, created_at DESC);

CREATE TRIGGER update_products_updated_at
	BEFORE UPDATE ON products
	FOR EACH ROW
	EXECUTE FUNCTION update_updated_at_column();
