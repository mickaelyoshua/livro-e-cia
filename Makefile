.PHONY: help docker-up docker-stop docker-start docker-down docker-logs docker-ps docker-clean db-setup db-migrate db-migrate-revert db-migrate-redo db-reset db-migration db-psql redis-cli gen-password gen-jwt backend-run frontend-run dev clean test test-unit test-integration test-db-setup

# Default target - show help
help:
	@echo "🔧 Livro & Cia - Development Commands"
	@echo ""
	@echo "📦 Docker Commands:"
	@echo "  make docker-up        - Start PostgreSQL and Redis containers"
	@echo "  make docker-stop      - Stop containers (fastest, keeps everything)"
	@echo "  make docker-start     - Resume stopped containers"
	@echo "  make docker-down      - Stop and remove containers (keep data)"
	@echo "  make docker-logs      - View container logs"
	@echo "  make docker-ps        - Show container status"
	@echo "  make docker-clean     - Stop and remove all data (⚠️  destructive)"
	@echo ""
	@echo "🗄️  Database Commands (Diesel):"
	@echo "  make db-setup         - Initialize Diesel and database"
	@echo "  make db-migrate       - Run pending migrations"
	@echo "  make db-migrate-redo  - Rollback and re-run last migration"
	@echo "  make db-reset         - Drop database and run all migrations"
	@echo "  make db-migration name=<name> - Create new migration"
	@echo "  make db-psql          - Open PostgreSQL shell"
	@echo ""
	@echo "🔐 Security Commands:"
	@echo "  make gen-password     - Generate strong password"
	@echo "  make gen-jwt          - Generate JWT secret"
	@echo ""
	@echo "🚀 Development Commands:"
	@echo "  make backend-run      - Run backend API server"
	@echo "  make frontend-run     - Run frontend dev server"
	@echo "  make dev              - Run backend and view logs"
	@echo ""
	@echo "🧪 Testing Commands:"
	@echo "  make test             - Run all tests (unit + integration)"
	@echo "  make test-unit        - Run unit tests only"
	@echo "  make test-integration - Run integration tests only"
	@echo "  make test-db-setup    - Create and migrate test database"
	@echo ""
	@echo "🧹 Cleanup:"
	@echo "  make clean            - Clean build artifacts"

# ==================== Docker Commands ====================

docker-up:
	@echo "🐳 Starting Docker containers..."
	docker compose up -d
	@echo "✅ Containers started. Waiting for health checks..."
	@sleep 3
	@docker compose ps

docker-stop:
	@echo "⏸️  Stopping Docker containers..."
	docker compose stop
	@echo "✅ Containers stopped (not removed, use 'make docker-start' to resume)"

docker-start:
	@echo "▶️  Starting stopped containers..."
	docker compose start
	@echo "✅ Containers started"
	@sleep 2
	@docker compose ps

docker-down:
	@echo "🛑 Stopping Docker containers..."
	docker compose down
	@echo "✅ Containers stopped (data preserved)"

docker-logs:
	@echo "📋 Showing container logs (Ctrl+C to exit)..."
	docker compose logs -f

docker-ps:
	@echo "📊 Container status:"
	@docker compose ps

docker-clean:
	@echo "⚠️  WARNING: This will delete all database data!"
	@echo "Press Ctrl+C to cancel, or wait 5 seconds to continue..."
	@sleep 5
	docker compose down -v
	@echo "✅ Containers and volumes removed"

# ==================== Database Commands ====================

db-setup:
	@echo "🗄️  Setting up Diesel..."
	cd backend && diesel setup
	@echo "✅ Diesel setup complete"

db-migrate:
	@echo "🔄 Running migrations..."
	cd backend && diesel migration run
	@echo "✅ Migrations complete"

db-migrate-revert:
	@echo "↩️  Undoing last migration..."
	cd backend && diesel migration revert
	@echo "✅ Migration undone"

db-migrate-redo:
	@echo "↩️  Redoing last migration..."
	cd backend && diesel migration redo
	@echo "✅ Migration redone"

db-reset:
	@echo "⚠️  WARNING: This will drop the database!"
	@echo "Press Ctrl+C to cancel, or wait 5 seconds to continue..."
	@sleep 5
	cd backend && diesel database reset
	@echo "✅ Database reset complete"

db-migration:
	@if [ -z "$(name)" ]; then \
		echo "❌ Error: Please provide migration name"; \
		echo "Usage: make db-migration name=create_users_table"; \
		exit 1; \
	fi
	@echo "📝 Creating migration: $(name)"
	cd backend && diesel migration generate $(name)
	@echo "✅ Migration files created in backend/migrations/"

db-psql:
	@echo "🐘 Connecting to PostgreSQL..."
	@echo "Tip: Use \\dt to list tables, \\q to quit"
	docker compose exec postgres psql -U livro_cia_user -d livro_cia_db

redis-cli:
	@echo "📮 Connecting to Redis..."
	@echo "Enter password when prompted"
	@read -p "Redis password: " REDIS_PASS; \
	docker compose exec livro_cia_redis redis-cli -a $$REDIS_PASS

# ==================== Security Commands ====================

gen-password:
	@echo "🔐 Generating strong password (URL-safe, no special chars):"
	@openssl rand -base64 32 | tr -d '+/='

gen-jwt:
	@echo "🔑 Generating JWT secret (256 bits):"
	@openssl rand -hex 32

# ==================== Development Commands ====================

backend-run:
	@echo "🚀 Starting backend server..."
	cd backend && cargo run

frontend-run:
	@echo "🎨 Starting frontend dev server..."
	cd frontend && trunk serve

dev: docker-up
	@echo "🚀 Starting development environment..."
	@echo "Backend will run on http://localhost:8000"
	@$(MAKE) backend-run

# ==================== Testing Commands ====================

# Test database URL - override with environment variable if needed
TEST_DB_URL ?= postgres://livro_cia_user:$(shell grep POSTGRES_PASSWORD .env 2>/dev/null | cut -d= -f2)@localhost:5432/livroecia_test

test-db-setup:
	@echo "🗄️  Setting up test database..."
	@echo "Creating database livroecia_test (if not exists)..."
	-docker compose exec postgres createdb -U livro_cia_user livroecia_test 2>/dev/null || true
	@echo "Running migrations on test database..."
	cd backend && DATABASE_URL="$(TEST_DB_URL)" diesel migration run
	@echo "✅ Test database ready"

test-unit:
	@echo "🧪 Running unit tests..."
	cd backend && cargo test --lib
	@echo "✅ Unit tests complete"

test-integration: test-db-setup
	@echo "🧪 Running integration tests (this may take a while)..."
	cd backend && TEST_DATABASE_URL="$(TEST_DB_URL)" cargo test --test '*' -- --test-threads=1
	@echo "✅ Integration tests complete"

test: test-db-setup
	@echo "🧪 Running all tests..."
	cd backend && TEST_DATABASE_URL="$(TEST_DB_URL)" cargo test -- --test-threads=1
	@echo "✅ All tests complete"

# ==================== Cleanup Commands ====================

clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "✅ Clean complete"
