CREATE TABLE roles (
	name VARCHAR(50) PRIMARY KEY,
	description TEXT
);

INSERT INTO roles (name, description) VALUES
	('admin', 'Full system access'),
	('manager', 'Store manager - reports and inventory'),
	('employee', 'Store employee - sales and basic inventory');
