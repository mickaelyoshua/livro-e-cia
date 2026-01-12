.PHONY: up down restart logs ps clean db-shell

# Start all services in detached mode
up:
	docker compose up -d

# Stop all services
down:
	docker compose down

# Restart all services
restart:
	docker compose restart

# View logs (follow mode)
logs:
	docker compose logs -f

# View logs for postgres only
logs-db:
	docker compose logs -f postgres

# Show running containers
ps:
	docker compose ps

# Stop and remove containers, networks, volumes
clean:
	docker compose down -v

# Open psql shell
db-shell:
	docker compose exec postgres psql -U $${POSTGRES_USER} -d $${POSTGRES_DB}

# Build and start (useful after Dockerfile changes)
build:
	docker compose up -d --build

# Check service health
health:
	docker compose ps --format "table {{.Name}}\t{{.Status}}"
