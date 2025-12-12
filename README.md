# Livro & Cia - Bookstore Management System

> Internal management system for "Livro & Cia" bookstore at PAZ Church João Pessoa

Full-stack web application built in Rust for managing bookstore operations: inventory control, sales tracking, and employee management. Designed exclusively for internal use by church staff.

## Tech Stack

### Backend (JSON API)
- **[Rust](https://www.rust-lang.org/)** with **[Rocket 0.5](https://rocket.rs/)** web framework
- **[Diesel 2.2](https://diesel.rs/)** ORM with **[PostgreSQL](https://www.postgresql.org/)**
- **[Redis](https://redis.io/)** for caching and sessions
- **[Tokio](https://tokio.rs/)** async runtime

### Frontend
- **[Leptos 0.7](https://leptos.dev/)** reactive framework
- **[WebAssembly](https://webassembly.org/)** for performance
- **[Trunk](https://trunkrs.dev/)** build tool

### Security & Auth
- Custom authentication (Argon2id + JWT)
- `jsonwebtoken` crate for token management
- `utoipa` for OpenAPI documentation

## Architecture

API-first design: backend is a pure JSON REST API, frontend is a separate WASM application.

```
Frontend (Leptos WASM) ←→ Backend (Rocket API) ←→ PostgreSQL/Redis
```

## Features

### Internal Management System
- Stock management (CRUD operations)
- Sales tracking and recording
- Employee management with role-based access
- Reports and analytics
- Audit logging
- Admin-managed user accounts
- Email verification and password reset

## Project Structure

```
livro-e-cia/
├── backend/           # Rocket API server
│   ├── src/
│   │   ├── routes/   # API endpoints
│   │   ├── models/   # Diesel models
│   │   ├── auth/     # Authentication
│   │   └── db/       # Database pool
│   └── migrations/   # Database migrations
│
├── frontend/          # Leptos WASM app
│   └── src/
│       ├── pages/    # Page components
│       ├── components/ # UI components
│       └── api/      # Backend client
│
└── shared/           # Shared types
```

## API Endpoints

### Authentication
- `POST /api/v1/auth/login` - Login (returns JWT)
- `POST /api/v1/auth/refresh` - Refresh token
- `POST /api/v1/auth/verify-email` - Verify employee email
- `POST /api/v1/auth/forgot-password` - Request password reset
- `GET /api/v1/auth/me` - Current user

*Note: Employee accounts are created by admins via `/api/v1/employees`*

### Products
- `GET /api/v1/products` - List (paginated)
- `GET /api/v1/products/:id` - Get single
- `POST /api/v1/products` - Create (admin)
- `PUT /api/v1/products/:id` - Update (admin)
- `DELETE /api/v1/products/:id` - Delete (admin)

### Inventory
- `GET /api/v1/inventory` - Stock levels
- `PUT /api/v1/inventory/:id` - Update stock
- `GET /api/v1/inventory/low` - Low stock alerts

### Sales
- `GET /api/v1/sales` - List (paginated)
- `POST /api/v1/sales` - Record sale
- `GET /api/v1/sales/:id` - Sale details
- `GET /api/v1/sales/report` - Generate report

### Employees
- `GET /api/v1/employees` - List (admin)
- `POST /api/v1/employees` - Create (admin)
- `PUT /api/v1/employees/:id` - Update (admin)
- `DELETE /api/v1/employees/:id` - Delete (admin)

## Development Setup

### Prerequisites

**Required:**
- Rust 1.75+
- Docker & Docker Compose (for databases)

**Install Tools:**
```bash
# Trunk - Leptos frontend build tool
cargo install trunk

# Diesel CLI - Database migration tool (PostgreSQL only)
cargo install diesel_cli --no-default-features --features postgres
```

**If Diesel CLI fails to compile**, install PostgreSQL development libraries:
```bash
# Arch Linux
sudo pacman -S postgresql-libs

# Ubuntu/Debian
sudo apt-get install libpq-dev

# macOS
brew install postgresql
```

**Verify installation:**
```bash
diesel --version  # Should show diesel 2.2.x
trunk --version
```

### Quick Start with Make

```bash
# Copy environment example and add your passwords
cp .env.example .env
# Edit .env with real passwords (use: make gen-password and make gen-jwt)

# Start databases
make docker-up

# Set up Diesel (first time only)
make db-setup

# See all available commands
make help
```

### Common Make Commands

```bash
# Docker operations
make docker-up        # Start PostgreSQL and Redis
make docker-down      # Stop containers (keep data)
make docker-logs      # View logs
make docker-ps        # Check status

# Database operations
make db-setup         # Initialize Diesel
make db-migrate       # Run migrations
make db-psql          # Open PostgreSQL shell

# Generate secure values
make gen-password     # For .env passwords
make gen-jwt          # For JWT secret

# Development
make backend-run      # Start backend API
make frontend-run     # Start frontend dev server
```

### Manual Commands (without Make)

**Backend:**
```bash
cd backend
diesel migration run
cargo run
```
API: `http://localhost:8000`
Swagger UI: `http://localhost:8000/swagger-ui/`

**Frontend:**
```bash
cd frontend
trunk serve
```
App: `http://localhost:8080`

## Testing

The project has two types of tests:

### Unit Tests

Test individual modules in isolation (JWT, password hashing, validation, etc.):

```bash
cd backend
cargo test --lib
```

### Integration Tests

Test full HTTP request/response cycles with real database:

**Prerequisites:**
1. PostgreSQL running (via `make docker-up`)
2. Test database created

**Setup (first time only):**
```bash
# Create test database
createdb livroecia_test

# Run migrations on test database
DATABASE_URL=postgres://livro_cia_user:your_password@localhost:5432/livroecia_test \
  diesel migration run
```

**Run Integration Tests:**
```bash
# Run all integration tests (serially - required for database isolation)
cd backend
TEST_DATABASE_URL=postgres://livro_cia_user:your_password@localhost:5432/livroecia_test \
  cargo test --test '*' -- --test-threads=1

# Run specific test file
cargo test --test auth_tests
cargo test --test products_tests
cargo test --test employees_tests
cargo test --test security_tests

# Run with output (see println! statements)
cargo test --test auth_tests -- --nocapture

# Run specific test by name
cargo test --test auth_tests login_valid_credentials
```

**Test Coverage:**

| Test File | Tests | Coverage |
|-----------|-------|----------|
| `auth_tests.rs` | 26 | Login, /me, refresh, logout, verify-email, password reset |
| `products_tests.rs` | 18 | CRUD, pagination, filters, admin-only, soft delete |
| `employees_tests.rs` | 22 | CRUD, pagination, admin-only, role changes |
| `security_tests.rs` | 13 | Auth bypass, token confusion, enumeration, injection |
| **Total** | **79** | All 17 API endpoints |

**Test Structure:**
```
backend/tests/
├── common/              # Shared test infrastructure
│   ├── mod.rs          # Module exports
│   ├── test_app.rs     # TestApp with Rocket client
│   ├── fixtures.rs     # Test data factories
│   └── auth_helpers.rs # JWT token generation
├── auth_tests.rs       # Authentication tests
├── products_tests.rs   # Product endpoint tests
├── employees_tests.rs  # Employee endpoint tests
└── security_tests.rs   # Security-focused tests
```

### Run All Tests

**Using Make (recommended):**
```bash
# Run all tests (unit + integration)
make test

# Run only unit tests
make test-unit

# Run only integration tests
make test-integration

# Setup test database (run once, or after schema changes)
make test-db-setup
```

**Manual commands:**
```bash
cd backend

# First, ensure test database exists and is migrated
TEST_DATABASE_URL=postgres://livro_cia_user:your_password@localhost:5432/livroecia_test \
  cargo test -- --test-threads=1
```

## Security Features

- Argon2id password hashing (OWASP recommended)
- JWT tokens (algorithm pinning, short expiration)
- Rate limiting on auth endpoints
- Input validation on all endpoints
- SQL injection prevention (Diesel ORM)
- XSS prevention (Leptos auto-escape)
- CORS protection
- Audit logging

## Roadmap

**Phase 1:** Core Internal Management
- [ ] Project setup and database schema
- [ ] Authentication (Argon2 + JWT)
- [ ] Product/inventory CRUD
- [ ] Sales tracking
- [ ] Employee management
- [ ] Reporting

**Phase 2:** Enhanced Features
- [ ] Advanced search/filtering
- [ ] Data export (CSV, PDF)
- [ ] Email notifications
- [ ] File uploads (book covers, receipts)
- [ ] Low stock alerts
- [ ] Advanced analytics

## License

Proprietary - PAZ Church João Pessoa

---

**Built with Rust**
