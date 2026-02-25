# ── Docker ──────────────────────────────────────────────
.PHONY: up
up:
	docker compose up -d

.PHONY: down
down:
	docker compose down

.PHONY: logs
logs:
	docker compose logs -f

.PHONY: health
health:
	docker inspect --format='{{.State.Health.Status}}' livro_cia_postgres

.PHONY: db-shell
db-shell:
	docker exec -it livro_cia_postgres psql -U $${POSTGRES_USER:-livro_cia} -d $${POSTGRES_DB:-livro_cia_db}

# ── Database (requires: cargo install sqlx-cli --no-default-features --features rustls,postgres) ──
.PHONY: db-setup
db-setup:
	sqlx database setup

.PHONY: db-migrate
db-migrate:
	sqlx migrate run

.PHONY: db-status
db-status:
	sqlx migrate info

.PHONY: db-reset
db-reset:
	sqlx database reset -y --force

# ── Development ─────────────────────────────────────────
.PHONY: build
build:
	cargo build

.PHONY: run
run:
	cargo run

.PHONY: dev
dev:
	cargo watch -x run

.PHONY: clippy
clippy:
	cargo clippy -- -D warnings

.PHONY: check
check:
	cargo check
