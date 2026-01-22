CREATE TABLE refresh_token_families (
	id UUID PRIMARY KEY,
	employee_id UUID NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
	current_jti UUID NOT NULL,
	is_revoked BOOLEAN NOT NULL DEFAULT FALSE,
	created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_refresh_token_families_employee ON refresh_token_families(employee_id);
CREATE INDEX idx_refresh_token_families_jti ON refresh_token_families(current_jti);
