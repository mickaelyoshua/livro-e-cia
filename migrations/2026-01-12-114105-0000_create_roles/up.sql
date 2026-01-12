CREATE TABLE roles (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	name VARCHAR(50) NOT NULL UNIQUE,
	description TEXT
);

INSERT INTO roles (name, description) VALUES
	('admin', 'Full system access'),
	('manager', 'Store manager - reports and inventory'),
	('employee', 'Store employee - sales and basic inventory');
