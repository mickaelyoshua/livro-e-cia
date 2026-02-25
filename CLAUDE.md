# CLAUDE.md - Livro e Cia

## Project Overview

Livro e Cia is a web application for internal management of a Christian bookstore. It handles **stock management**, **sales transactions**, and **statistical analysis of sales data**. The application targets the Brazilian market (Portuguese content, PIX payment method).

## AI Assistant Guidelines

- **Do NOT modify code** unless explicitly requested
- **Explain and show** what changes are needed - the developer will implement them
- Provide code snippets, file paths, and clear instructions
- Ask clarifying questions before suggesting solutions

## Design Principles

### Simple but Complete
- Keep the codebase minimal and focused - no over-engineering
- Implement only what is needed, but implement it fully
- Avoid unnecessary abstractions and premature optimization
- Prefer straightforward solutions over clever ones
- Each feature should be complete and production-ready

### Security First
- Validate all input at API boundaries
- Use parameterized queries (SQLx handles this)
- Hash passwords with Argon2id (OWASP-recommended params)
- Implement proper JWT validation and expiration
- Apply principle of least privilege for role-based access
- Sanitize all user-provided data before use
- Never expose sensitive information in error messages
- Use HTTPS in production

### Idiomatic Rust
- Leverage the type system for correctness
- Use `Result<T, E>` for error handling - no panics in business logic
- Prefer `Option<T>` over null/sentinel values
- Use enums for state machines and variants
- Implement `From`/`Into` traits for type conversions
- Use iterators and functional patterns where appropriate
- Follow Rust API guidelines (https://rust-lang.github.io/api-guidelines/)
- Let the compiler help you - embrace strict typing
- Use `clippy` for linting and follow its suggestions

## Architecture

### Response Format: HTML (HTMX + Askama templates)

This is a **monolithic server-rendered application**. Routes return HTML, not JSON.

**Why HTMX:**
- True REST (hypermedia) - self-describing responses
- Single deployment - Rust backend serves everything
- Fast development - no client state management
- Proven, stable technology
- Future-proof: JSON endpoints can be added later if needed

**Why Askama:**
- Compile-time template checking - catches errors at build time, not runtime
- Type-safe - templates reference Rust structs directly
- Fast - zero runtime overhead, templates compile to Rust code

### Response Guidelines

Routes should return:
- **Full HTML pages** for initial page loads
- **HTML fragments** for HTMX requests (check `HX-Request` header)
- Use Askama templates with `#[derive(Template, WebTemplate)]`

```rust
use askama::Template;
use askama_web::WebTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "products/list.html")]
struct ProductsPage {
    products: Vec<ProductView>,
}

async fn products_page() -> ProductsPage {
    // Template implements IntoResponse automatically via WebTemplate
    ProductsPage { products }
}
```

## Tech Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Language | Rust | Edition 2024 |
| Web Framework | Axum | 0.8 |
| Templates | Askama | 0.15 (compile-time) + askama_web 0.15 |
| Frontend | HTMX | 2.0.4 (vendored) |
| Charts | Chart.js | (future) |
| Database | PostgreSQL | 16 (Alpine) |
| DB Toolkit | SQLx | 0.8 (async, compile-time checked queries) |
| Auth | JWT (jsonwebtoken 10.2) + Argon2id (argon2 0.5) |
| Cookies | axum-extra PrivateCookieJar (encrypted) |
| Async Runtime | tokio | 1 |
| Containers | Docker Compose | - |

### Dependencies (Cargo.toml)

**Web Framework:**
- `axum` 0.8 - Async web framework (with `macros` for `#[debug_handler]`)
- `axum-extra` 0.12 - Encrypted private cookies (`cookie-private`)
- `tokio` 1 - Async runtime (`full` features)
- `tower-http` 0.6 - HTTP middleware (`fs` for static files, `trace` for logging)
- `serde` 1.0 - Serialization (`derive`)

**Templates:**
- `askama` 0.15 - Compile-time template engine
- `askama_web` 0.15 - Axum integration (`axum-0.8` feature)

**Database:**
- `sqlx` 0.8 - Async PostgreSQL toolkit (`postgres`, `runtime-tokio`, `tls-rustls`, `uuid`, `chrono`, `rust_decimal`, `migrate`)
- `uuid` 1.19 - UUID v4 generation (`v4`, `serde`)
- `rust_decimal` 1.39 - Monetary precision (`serde-with-str`)
- `chrono` 0.4 - Date/time handling (`serde`)

**Authentication:**
- `jsonwebtoken` 10.2 - JWT tokens (`rust_crypto`)
- `argon2` 0.5 - Password hashing (Argon2id)
- `rand` 0.10 - Cryptographic random generation

**Utilities:**
- `dotenvy` 0.15 - Environment variables
- `thiserror` 2.0 - Error handling
- `tracing` 0.1 / `tracing-subscriber` 0.3 - Structured logging (`env-filter`)

## Project Structure

```
livro-e-cia/
├── src/
│   ├── main.rs                   # Axum bootstrap (PgPool, migrations, Router, TcpListener)
│   ├── config.rs                 # AppConfig (from env vars)
│   ├── error.rs                  # AppError enum with IntoResponse impl
│   ├── auth/
│   │   ├── mod.rs                # Module exports
│   │   ├── claims.rs             # AccessClaims (15min) + RefreshClaims (7d) with rotation
│   │   ├── cookie.rs             # PrivateCookieJar management (set/get/remove)
│   │   ├── extractors.rs         # Axum extractors (AuthenticatedEmployee, AdminOnly, ManagerOrAbove)
│   │   ├── password.rs           # Argon2id hash/verify (OWASP params)
│   │   └── tokens.rs             # JwtConfig: generate, validate, rotate access/refresh tokens
│   ├── models/
│   │   ├── mod.rs                # Module exports
│   │   ├── role.rs               # Role (PK: name VARCHAR)
│   │   ├── employee.rs           # Employee + NewEmployee + UpdateEmployee
│   │   ├── category.rs           # Category + NewCategory + UpdateCategory
│   │   ├── product.rs            # Product + NewProduct + UpdateProduct
│   │   ├── sale.rs               # Sale + NewSale
│   │   ├── sale_item.rs          # SaleItem + NewSaleItem (composite PK)
│   │   └── payment_method.rs     # PaymentMethod enum with sqlx::Type
│   ├── repositories/
│   │   ├── mod.rs                # Module exports
│   │   ├── employee_repo.rs      # Employee CRUD
│   │   ├── product_repo.rs       # Product CRUD
│   │   ├── category_repo.rs      # Category CRUD
│   │   ├── sale_repo.rs          # Sale + SaleItem operations
│   │   └── refresh_token_repo.rs # Token family tracking
│   └── routes/
│       ├── mod.rs                # Module exports + Router assembly
│       ├── auth.rs               # Login, logout, refresh
│       ├── employees.rs          # Employee management
│       ├── products.rs           # Product CRUD
│       ├── categories.rs         # Category CRUD
│       ├── sales.rs              # Sale operations
│       ├── stats.rs              # Statistics/reports
│       └── stock.rs              # Stock management
├── migrations/                   # SQLx migrations (flat .sql files)
│   ├── 20260112114105_create_roles.sql
│   ├── 20260112114424_create_employees.sql
│   ├── 20260112115846_create_categories.sql
│   ├── 20260112120045_create_products.sql
│   ├── 20260112125137_create_payment_method_enum.sql
│   ├── 20260112125347_create_sales.sql
│   ├── 20260112125600_create_sale_items.sql
│   ├── 20260112125655_create_stock_trigger.sql
│   ├── 20260115151000_create_refresh_token_families.sql
│   └── 20260219000000_add_partial_indexes.sql
├── templates/                    # Askama templates (compiled at build time)
│   ├── base.html                 # Base layout (includes HTMX script)
│   └── error.html                # Error page
├── static/
│   └── js/
│       └── htmx.min.js           # HTMX 2.0.4 (vendored)
├── Cargo.toml
├── docker-compose.yml            # PostgreSQL 16 container
├── Makefile                      # Development commands
├── .env-example                  # Environment template
└── .env                          # Environment variables (in .gitignore)
```

## Database Schema

### Entity Relationship

```
roles (PK: name, 3 seed records: admin, manager, employee)
  │
  └──< employees (FK: role -> roles.name)
         │
         ├──< sales ──< sale_items >── products >── categories (8 seed records)
         │
         └──< refresh_token_families (token rotation tracking)
```

### Tables

| Table | Purpose | Key Fields |
|-------|---------|------------|
| `roles` | User roles | **name** (PK, VARCHAR), description |
| `employees` | User accounts | id (UUID), email (unique), password_hash, name, **role** (VARCHAR FK -> roles.name), is_active |
| `categories` | Book categories | id (UUID), name (unique), description, created_at, updated_at |
| `products` | Book inventory | id (UUID), title, author, price, stock_quantity, publisher, publication_date, category_id, description, cover_image_url, is_active |
| `sales` | Transactions | id (UUID), seller_id, subtotal, discount, total, payment_method (ENUM), notes |
| `sale_items` | Line items | (sale_id, product_id) composite PK, quantity, unit_price, subtotal |
| `refresh_token_families` | JWT refresh rotation | id (UUID), employee_id, current_jti, is_revoked, created_at, last_used_at |

### Payment Methods (PostgreSQL ENUM)
- `cash`, `credit_card`, `debit_card`, `pix`

### CHECK Constraints

| Table | Constraint | Rule |
|-------|-----------|------|
| `sales` | `check_total_consistency` | `total = subtotal - discount` |
| `sale_items` | `check_subtotal_consistency` | `subtotal = quantity * unit_price` |
| `employees` | `check_name_not_empty` | `LENGTH(TRIM(name)) > 0` |
| `products` | (inline) | `price >= 0`, `stock_quantity >= 0` |
| `sales` | (inline) | `subtotal >= 0`, `discount >= 0`, `total >= 0` |
| `sale_items` | (inline) | `quantity > 0`, `unit_price >= 0`, `subtotal >= 0` |

### Database Triggers

| Trigger | Table | Purpose |
|---------|-------|---------|
| `update_employees_updated_at` | employees | Auto-update `updated_at` on UPDATE |
| `update_categories_updated_at` | categories | Auto-update `updated_at` on UPDATE |
| `update_products_updated_at` | products | Auto-update `updated_at` on UPDATE |
| `update_sales_updated_at` | sales | Auto-update `updated_at` on UPDATE |
| `trg_decrease_stock` | sale_items | Auto-decrement product stock on INSERT |

**Important:** Stock is managed by database triggers - do NOT manually update stock when processing sales.

### Partial Indexes

| Index | Table | Filter | Purpose |
|-------|-------|--------|---------|
| `idx_employees_active` | employees | `WHERE is_active = TRUE` | Fast lookup of active employees |
| `idx_products_active` | products | `WHERE is_active = TRUE` | Fast lookup of active products |
| `idx_refresh_token_active` | refresh_token_families | `WHERE is_revoked = FALSE` | Fast lookup of valid token families |

### Seed Data

**Roles:**
- `admin` - Full system access
- `manager` - Store manager - reports and inventory
- `employee` - Store employee - sales and basic inventory

**Categories (Portuguese):**
- Biblias, Estudos Biblicos, Devocionais, Vida Crista
- Familia e Relacionamentos, Infantil, Teologia, Biografias

## Data Models

### Core Models (src/models/)

All models derive `sqlx::FromRow` and `serde::Serialize` for database mapping and template rendering.

```rust
// Pattern for each entity:
#[derive(sqlx::FromRow, Serialize)]
pub struct Entity { ... }           // FromRow - read from DB

pub struct NewEntity { ... }        // Used in INSERT queries

pub struct UpdateEntity { ... }     // Used in UPDATE queries (Option<T> fields for partial updates)
```

**Note:** `employees.role` is a `String` (VARCHAR FK to `roles.name`), NOT a UUID.

### PaymentMethod Enum

```rust
#[derive(sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "payment_method", rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    CreditCard,
    DebitCard,
    Pix,
}
```

SQLx maps this directly to the PostgreSQL `payment_method` ENUM via the `sqlx::Type` derive.

## Authentication System

### Architecture

Uses **access + refresh token rotation** with **encrypted private cookies** (`axum-extra` `PrivateCookieJar`).

| Token | TTL | Storage | Purpose |
|-------|-----|---------|---------|
| Access token | 15 minutes | Private cookie (path `/`) | API authentication |
| Refresh token | 7 days | Private cookie (path `/auth`) | Token rotation |

### Refresh Token Families

Implements **token family tracking** for replay attack detection:
- Each refresh creates a new `jti` within the same `family_id`
- Reuse of an old `jti` revokes the entire family (`TokenReuse` error)
- Tracked in `refresh_token_families` table

### Axum Extractors (src/auth/extractors.rs)

Authentication uses Axum's `FromRequestParts` trait to create extractors:

```rust
// Usage in route handlers:
async fn list_employees(auth: AuthenticatedEmployee) -> impl IntoResponse { ... }
async fn create_employee(auth: AdminOnly, ...) -> impl IntoResponse { ... }
async fn update_product(auth: ManagerOrAbove, ...) -> impl IntoResponse { ... }
```

| Extractor | Purpose |
|-----------|---------|
| `AuthenticatedEmployee` | Validates access token cookie, returns claims |
| `AdminOnly` | Requires `admin` role |
| `ManagerOrAbove` | Requires `admin` or `manager` role |

### Password Hashing (src/auth/password.rs)

- Algorithm: Argon2id
- OWASP-recommended parameters: 19MiB memory, 2 iterations, 1 parallelism thread
- Functions: `hash_password()`, `verify_password()`

## Error Handling

`AppError` enum in `src/error.rs` implements `axum::response::IntoResponse`:

```rust
pub enum AppError {
    // Infrastructure
    Database(sqlx::Error),
    Internal(String),
    // Auth
    Unauthorized,
    Forbidden,
    InvalidCredentials,
    TokenExpired,
    TokenReuse,
    // Resource
    NotFound,
    Validation(String),
}
```

- Maps to appropriate HTTP status codes via `IntoResponse`
- Renders `error.html` Askama template
- Logs full errors server-side via `tracing::error!`
- Returns safe user messages (never exposes DB internals)

## Development Commands

```bash
# Docker
make up              # Start PostgreSQL container
make down            # Stop containers
make logs            # View container logs
make db-shell        # Connect to PostgreSQL shell
make health          # Check container health

# Database (requires sqlx-cli: cargo install sqlx-cli --features postgres)
make db-migrate      # Run pending migrations
make db-revert       # Revert last migration
make db-reset        # Drop and recreate database, run all migrations

# Build
cargo build          # Build the project
cargo run            # Run the server
cargo test           # Run tests
cargo clippy         # Run linter
```

## Environment Variables

Required in `.env`:

```bash
DATABASE_URL=postgresql://user:pass@localhost:5432/livro_cia_db
JWT_SECRET=<minimum-32-bytes-hex-secret>
HOST=127.0.0.1
PORT=8000
COOKIE_SECRET_KEY=<64-byte-hex-for-encrypted-cookies>
RUST_LOG=info
APP_ENV=development
```

## Implementation Status

### Completed
- [x] Cargo.toml with new dependency stack (Axum, SQLx, Askama)
- [x] Database schema (7 tables + payment_method ENUM)
- [x] All migrations (10 total, SQLx format)
- [x] Database relationships and constraints
- [x] CHECK constraints on derived values (sales.total, sale_items.subtotal)
- [x] Stock management trigger (automatic on sale_items INSERT)
- [x] Timestamp triggers (auto-updated_at on employees, categories, products, sales)
- [x] Partial indexes (active employees, active products, valid token families)
- [x] Docker Compose PostgreSQL setup

### Not Implemented (TODO)
- [ ] **src/main.rs** - Axum bootstrap (PgPool, migrations, Router, static files)
- [ ] **HTMX library** - Download vendored htmx.min.js
- [ ] **All data models** - Role, Employee, Category, Product, Sale, SaleItem, PaymentMethod
- [ ] **AppError** with IntoResponse
- [ ] **AppConfig** from env vars
- [ ] **Auth extractors** - AuthenticatedEmployee, AdminOnly, ManagerOrAbove
- [ ] **JWT token** generation, validation, rotation
- [ ] **Password hashing** (Argon2id)
- [ ] **Cookie management** (PrivateCookieJar)
- [ ] **All repositories** - employee, product, category, sale, refresh_token
- [ ] **All route handlers** - auth, employees, products, categories, sales, stats, stock
- [ ] **HTML templates** - base.html, error.html, and all page templates
- [ ] **Makefile** - Development commands
- [ ] **.env-example** - Updated environment template
- [ ] **Input validation** at API boundaries
- [ ] **Tests**

## API Design Guidelines

### Expected Endpoints

**Authentication**
- `POST /auth/login` - User login
- `POST /auth/logout` - User logout
- `POST /auth/refresh` - Token rotation

**Employees**
- `GET /employees` - List employees
- `GET /employees/{id}` - Get employee details
- `POST /employees` - Create employee (admin)
- `PUT /employees/{id}` - Update employee (admin)
- `DELETE /employees/{id}` - Deactivate employee (admin)

**Products**
- `GET /products` - List products (with pagination, filtering)
- `GET /products/{id}` - Get product details
- `POST /products` - Create product (manager+)
- `PUT /products/{id}` - Update product (manager+)
- `DELETE /products/{id}` - Soft delete product (admin)

**Categories**
- `GET /categories` - List all categories
- `POST /categories` - Create category (admin)

**Sales**
- `POST /sales` - Create new sale (auto-decrements stock via trigger)
- `GET /sales` - List sales (with date filtering)
- `GET /sales/{id}` - Get sale details with items

**Statistics**
- `GET /stats/sales` - Sales statistics (daily, weekly, monthly)
- `GET /stats/products` - Top selling products
- `GET /stats/categories` - Sales by category

**Stock**
- `GET /stock/low` - Products with low stock
- `PUT /stock/{product_id}` - Adjust stock manually (manager+)

## Role-Based Access

| Role | Permissions |
|------|-------------|
| admin | Full access, user management, system configuration |
| manager | Product CRUD, stock management, view all sales |
| employee | Create sales, view own sales, view products |

## Coding Conventions

- Use `snake_case` for functions and variables
- Use `PascalCase` for structs and enums
- Handle errors with `Result<T, E>` - no unwrap in business logic
- Use SQLx for all database operations (raw SQL via `sqlx::query` / `sqlx::query_as`)
- Validate input at API boundaries
- Use Axum extractors for authentication/authorization
- All monetary values use `rust_decimal::Decimal`
- All IDs use `uuid::Uuid`
- All timestamps use `chrono::DateTime<Utc>`
- Database pool: pass `PgPool` via Axum `State`
- Templates: derive both `askama::Template` and `askama_web::WebTemplate`
- HTMX: vendored at `static/js/htmx.min.js`, served via `tower_http::services::ServeDir`

## Notes

- Stock is automatically managed via database triggers - do NOT manually update stock when processing sales
- All monetary values use `DECIMAL(10,2)` for precision - stored as-is, NOT as multiplied integers
- UUIDs are used for all primary keys (except `roles` which uses `name` VARCHAR as PK)
- `employees.role` is a VARCHAR FK to `roles.name`, NOT a UUID foreign key
- SQLx migrations are flat `.sql` files in `migrations/` - no up/down separation
- Askama templates are compiled at build time - template errors are caught during `cargo build`
- HTMX is vendored locally (no CDN dependency at runtime)
