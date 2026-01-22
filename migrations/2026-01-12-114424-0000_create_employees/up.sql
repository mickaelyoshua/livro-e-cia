CREATE TABLE employees (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	email VARCHAR(255) NOT NULL,
	password_hash VARCHAR(255) NOT NULL,
	name VARCHAR(255) NOT NULL,
	role VARCHAR(50) NOT NULL REFERENCES roles(name) ON DELETE RESTRICT,
	is_active BOOLEAN NOT NULL DEFAULT TRUE,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	CONSTRAINT check_name_not_empty CHECK (LENGTH(TRIM(name)) > 0)
);

CREATE UNIQUE INDEX idx_employees_email ON employees(LOWER(email));
CREATE INDEX idx_employees_role ON employees(role);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
	NEW.updated_at = NOW();
	RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_employees_updated_at
	BEFORE UPDATE ON employees
	FOR EACH ROW
	EXECUTE FUNCTION update_updated_at_column();
