CREATE TABLE categories (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	name VARCHAR(100) NOT NULL UNIQUE,
	description TEXT,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_categories_name ON categories(name);

CREATE TRIGGER update_categories_updated_at
	BEFORE UPDATE ON categories
	FOR EACH ROW
		EXECUTE FUNCTION update_updated_at_column();
