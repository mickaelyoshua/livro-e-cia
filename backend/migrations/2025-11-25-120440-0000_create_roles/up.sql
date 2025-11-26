CREATE TABLE roles (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	name VARCHAR(50) NOT NULL UNIQUE,
	description TEXT,
	created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_roles_name ON roles(name);

-- Seed default roles
INSERT INTO roles (name, description)
VALUES
	('admin', 'Full system access'),
	('manager', 'Store manager - reports and inventory'),
	('employee', 'Store employee - sales and basic inventory');

-- Auto-update trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$ -- designed to be called by triggers only
BEGIN
	-- NEW is the row being inserted or updated
	NEW.updated_at = NOW();
	RETURN NEW;
END;
-- PL/pgSQL = PostgreSQL's procedural language (SQL + programming)
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_roles_updated_at
	-- run BEFORE PostgreSQL saves changes | only runs on UPDATE queries | ONly for roles table
	BEFORE UPDATE ON roles
	FOR EACH ROW
		EXECUTE FUNCTION update_updated_at_column();
