# Livro e Cia

Internal management system for a Christian bookstore. Handles stock management, sales transactions, and sales analytics.

Built as a server-rendered monolith with Rust, HTMX, and PostgreSQL.

## Tech Stack

- **Backend:** Rust (Edition 2024) with [Axum](https://github.com/tokio-rs/axum) 0.8
- **Templates:** [Askama](https://github.com/djc/askama) (compile-time checked)
- **Frontend:** [HTMX](https://htmx.org/) 2.0.4 (vendored)
- **Database:** PostgreSQL 16 with [SQLx](https://github.com/launchbadge/sqlx) 0.8
- **Auth:** JWT access/refresh token rotation + Argon2id password hashing
- **Cookies:** Encrypted private cookies via `axum-extra`

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Docker](https://docs.docker.com/get-docker/) and Docker Compose
- [sqlx-cli](https://crates.io/crates/sqlx-cli):
  ```bash
  cargo install sqlx-cli --no-default-features --features rustls,postgres
  ```

## Getting Started

1. **Clone and configure:**
   ```bash
   git clone https://github.com/your-user/livro-e-cia.git
   cd livro-e-cia
   cp .env-example .env
   # Edit .env — update JWT_SECRET and COOKIE_SECRET_KEY with real secrets
   ```

2. **Start PostgreSQL:**
   ```bash
   make up
   ```

3. **Run database migrations:**
   ```bash
   make db-migrate
   ```

4. **Run the application:**
   ```bash
   cargo run
   ```

   The server starts at `http://127.0.0.1:8000` by default.

## Development Commands

```bash
# Docker
make up           # Start PostgreSQL container
make down         # Stop containers
make logs         # Tail container logs
make health       # Check PostgreSQL health
make db-shell     # Open psql shell

# Database
make db-migrate   # Run pending migrations
make db-reset     # Drop, recreate, and migrate (destructive)
make db-status    # Show migration status

# Build & Run
make build        # cargo build
make run          # cargo run
make dev          # cargo watch -x run (live reload)
make clippy       # Lint with clippy
make check        # cargo check
```

## Project Structure

```
src/
├── main.rs              # Axum bootstrap
├── config.rs            # AppConfig from env vars
├── error.rs             # AppError with IntoResponse
├── auth/                # JWT, password hashing, cookie management, extractors
├── models/              # Data models (Role, Employee, Product, Sale, etc.)
└── repositories/        # Database operations (CRUD per entity)

migrations/              # SQLx migrations (10 total)
templates/               # Askama HTML templates
static/js/               # Vendored HTMX
```

## Database

PostgreSQL 16 with 7 tables:

| Table | Purpose |
|-------|---------|
| `roles` | 3 roles: admin, manager, employee |
| `employees` | User accounts with role-based access |
| `categories` | 8 seeded book categories |
| `products` | Book inventory with stock tracking |
| `sales` | Sales transactions |
| `sale_items` | Line items per sale |
| `refresh_token_families` | JWT refresh token rotation |

Stock is managed automatically via database triggers — inserting sale items decrements product stock.

## Authentication

- **Access tokens** (15 min) and **refresh tokens** (7 days) stored in encrypted cookies
- Refresh token family tracking detects replay attacks
- Passwords hashed with Argon2id (OWASP-recommended parameters)
- Role-based access: `AdminOnly`, `ManagerOrAbove`, `AuthenticatedEmployee` extractors

## Environment Variables

See [`.env-example`](.env-example) for all required variables:

| Variable | Description |
|----------|-------------|
| `DATABASE_URL` | PostgreSQL connection string |
| `JWT_SECRET` | Minimum 32 bytes for token signing |
| `COOKIE_SECRET_KEY` | 64 bytes for cookie encryption |
| `HOST` / `PORT` | Server bind address (default `127.0.0.1:8000`) |
| `APP_ENV` | `development` or `production` |
| `RUST_LOG` | Log level filter (default `info`) |

## License

[MIT](LICENSE) — Mickael Yoshua
