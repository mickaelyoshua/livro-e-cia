CREATE TABLE users (
	id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
	email VARCHAR(255) NOT NULL UNIQUE,
	password_hash VARCHAR(255) NOT NULL,
	name VARCHAR(255) NOT NULL,
	role_id UUID NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
	is_active BOOLEAN NOT NULL DEFAULT TRUE,
	email_verified BOOLEAN NOT NULL DEFAULT FALSE,
	email_verified_at TIMESTAMP WITH TIME ZONE,
	password_reset_token VARCHAR(255),
	password_reset_expires_at TIMESTAMP WITH TIME ZONE,
	created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
	updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
	last_login_at TIMESTAMP WITH TIME ZONE
);

-- Indexes
CREATE UNIQUE INDEX idx_users_email on users(LOWER(email));
CREATE INDEX idx_users_role_id ON users(role_id);
CREATE INDEX idx_users_is_active ON users(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_users_password_reset_token ON users(password_reset_token)
	WHERE password_reset_token IS NOT NULL;

-- Trigger
CREATE TRIGGER update_users_updated_at
	BEFORE UPDATE ON users
	FOR EACH ROW
		EXECUTE FUNCTION update_updated_at_column();

-- Constraints
ALTER TABLE users ADD CONSTRAINT check_email_format
	CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$');

ALTER TABLE users ADD CONSTRAINT check_password_hash_length
	CHECK (LENGTH(password_hash) >= 50);

ALTER TABLE users ADD CONSTRAINT check_name_not_empty
	CHECK (LENGTH(TRIM(name)) > 0);
