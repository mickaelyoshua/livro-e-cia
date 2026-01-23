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
- Use parameterized queries (Diesel handles this)
- Hash passwords with Argon2 or bcrypt
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

### Response Format: HTML (HTMX + Tera templates)

This is a **monolithic server-rendered application**. Routes return HTML, not JSON.

**Why HTMX:**
- True REST (hypermedia) - self-describing responses
- Single deployment - Rust backend serves everything
- Fast development - no client state management
- Proven, stable technology
- Future-proof: JSON endpoints can be added later if needed

### Response Guidelines

Routes should return:
- **Full HTML pages** for initial page loads
- **HTML fragments** for HTMX requests (check `HX-Request` header)
- Use `rocket_dyn_templates::Template` for rendering

```rust
#[get("/products")]
fn products_page(hx: Option<HxRequest>) -> Template {
    // Return full page or fragment based on HX-Request header
}
```

## Tech Stack

| Component | Technology | Version |
|-----------|------------|---------|
| Language | Rust | Edition 2024 |
| Web Framework | Rocket | 0.5 |
| Templates | Tera | via rocket_dyn_templates 0.2 |
| Frontend | HTMX + Chart.js | - |
| Database | PostgreSQL | 16 (Alpine) |
| ORM | Diesel | 2.3 |
| Authentication | JWT | (planned) |
| Containers | Docker Compose | - |

### Dependencies (Cargo.toml)

**Database & ORM:**
- `diesel` 2.3 - PostgreSQL ORM with r2d2, chrono, uuid, numeric
- `diesel_migrations` 2.3 - Migration runner
- `uuid` 1.19 - UUID v4 generation
- `rust_decimal` 1.39 - Monetary precision
- `chrono` 0.4 - Date/time handling

**Web Framework:**
- `rocket` 0.5 - Web framework
- `rocket_dyn_templates` 0.2 - Tera template rendering
- `serde` 1.0 - Serialization

**Utilities:**
- `dotenvy` 0.15 - Environment variables
- `thiserror` 2.0 - Error handling
- `tracing` 0.1 / `tracing-subscriber` 0.3 - Structured logging

## Project Structure

```
livro-e-cia/
├── src/
│   ├── main.rs                   # Entry point (placeholder - needs Rocket integration)
│   ├── db.rs                     # Database connection pooling (r2d2, max 10 connections)
│   ├── error.rs                  # AppError enum with Responder impl
│   ├── schema.rs                 # Diesel auto-generated (DO NOT EDIT)
│   ├── models/
│   │   ├── mod.rs                # Module exports
│   │   ├── role.rs               # Role model
│   │   ├── employees.rs          # Employee + NewEmployee + UpdateEmployee
│   │   ├── category.rs           # Category + NewCategory + UpdateCategory
│   │   ├── product.rs            # Product + NewProduct + UpdateProduct
│   │   ├── sale.rs               # Sale + NewSale + UpdateSale
│   │   ├── sale_item.rs          # SaleItem + NewSaleItem (composite PK)
│   │   ├── payment_method.rs     # PaymentMethod enum with custom Diesel impl
│   │   ├── forms_models.rs       # Form request structs + FormDecimal/FormNaiveDate helpers
│   │   └── views_models.rs       # Response DTOs (EmployeeView, SaleView)
│   └── routes/
│       ├── mod.rs                # Module exports
│       └── employees.rs          # Employee routes (EMPTY - not implemented)
├── migrations/                   # 9 Diesel migrations (all applied)
│   ├── 2026-01-12-*_create_roles/
│   ├── 2026-01-12-*_create_employees/
│   ├── 2026-01-12-*_create_categories/
│   ├── 2026-01-12-*_create_products/
│   ├── 2026-01-12-*_create_payment_method_enum/
│   ├── 2026-01-12-*_create_sales/
│   ├── 2026-01-12-*_create_sale_items/
│   └── 2026-01-12-*_create_stock_trigger/
├── templates/
│   └── base.html                 # Base template (EMPTY - not implemented)
├── static/js/
│   └── htmx.min.js               # HTMX library
├── Cargo.toml
├── Rocket.toml                   # Server config (port 8000, 4 workers dev, 16 release)
├── diesel.toml
├── docker-compose.yml            # PostgreSQL 16 container
├── Makefile                      # Development commands
└── .env                          # Environment variables
```

## Database Schema

### Entity Relationship

```
roles (3 seed records: admin, manager, employee)
  │
  └──< employees
         │
         └──< sales ──< sale_items >── products >── categories (8 seed records)
```

### Tables

| Table | Purpose | Key Fields |
|-------|---------|------------|
| `roles` | User roles | id, name, description |
| `employees` | User accounts | id, email (unique), password_hash, name, role_id, is_active |
| `categories` | Book categories | id, name (unique), description |
| `products` | Book inventory | id, title, author, price, stock_quantity, category_id, is_active |
| `sales` | Transactions | id, seller_id, subtotal, discount, total, payment_method, notes |
| `sale_items` | Line items | (sale_id, product_id) PK, quantity, unit_price, subtotal |

### Payment Methods (PostgreSQL ENUM)
- `cash`, `credit_card`, `debit_card`, `pix`

### Database Triggers

| Trigger | Table | Purpose |
|---------|-------|---------|
| `update_employees_updated_at` | employees | Auto-update `updated_at` on UPDATE |
| `update_products_updated_at` | products | Auto-update `updated_at` on UPDATE |
| `update_sales_updated_at` | sales | Auto-update `updated_at` on UPDATE |
| `trg_decrease_stock` | sale_items | Auto-decrement product stock on INSERT |

**Important:** Stock is managed by database triggers - do NOT manually update stock when processing sales.

### Seed Data

**Roles:**
- `admin` - Full system access
- `manager` - Store manager - reports and inventory
- `employee` - Store employee - sales and basic inventory

**Categories (Portuguese):**
- Bíblias, Estudos Bíblicos, Devocionais, Vida Cristã
- Família e Relacionamentos, Infantil, Teologia, Biografias

## Data Models

### Core Models (src/models/)

All models derive `Queryable`, `Identifiable`, `Serialize`. Insertable structs use `Insertable` derive. Update structs use `AsChangeset` derive.

```rust
// Pattern for each entity:
pub struct Entity { ... }           // Queryable - read from DB
pub struct NewEntity { ... }        // Insertable - create new
pub struct UpdateEntity { ... }     // AsChangeset - partial updates with Option<T>
```

### Form Helpers (forms_models.rs)

Custom wrapper types for form field parsing:

```rust
pub struct FormDecimal(pub rust_decimal::Decimal);  // Parses string to Decimal
pub struct FormNaiveDate(pub NaiveDate);            // Parses YYYY-MM-DD to NaiveDate
```

### View Models (views_models.rs)

DTOs for template rendering with joined data:

```rust
pub struct EmployeeView { id, email, name, role_name, created_at, updated_at }
pub struct SaleView { id, seller_name, subtotal, discount, total, payment_method, notes, created_at, updated_at, item_count }
```

## Error Handling

`AppError` enum in `src/error.rs` implements `Responder`:

```rust
pub enum AppError {
    Database(diesel::result::Error),
    Pool(diesel::r2d2::Error),
    Unauthorized,
    Forbidden,
    NotFound,
    Validation(String),
}
```

- Maps to appropriate HTTP status codes
- Renders `error.html` template
- Logs full errors server-side
- Returns safe user messages (never exposes DB internals)

## Development Commands

```bash
# Docker
make up              # Start PostgreSQL container
make down            # Stop containers
make logs            # View container logs
make db-shell        # Connect to PostgreSQL shell
make health          # Check container health

# Database
diesel migration run           # Run pending migrations
diesel migration revert        # Revert last migration
diesel migration generate NAME # Create new migration

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
JWT_SECRET=<256-bit-hex-secret>
ROCKET_PORT=8000
ROCKET_ADDRESS=127.0.0.1
RUST_LOG=info
```

## Implementation Status

### Completed
- [x] Database schema (6 tables + payment_method ENUM)
- [x] All migrations (9 total, including triggers)
- [x] Database relationships and constraints
- [x] Stock management trigger (automatic on sale_items INSERT)
- [x] Timestamp triggers (auto-updated_at)
- [x] All data models (Role, Employee, Category, Product, Sale, SaleItem)
- [x] Payment method with custom Diesel ENUM serialization
- [x] Form request models with custom field types
- [x] View/response models (EmployeeView, SaleView)
- [x] Error handling (AppError with HTTP responses)
- [x] Database connection pooling (r2d2)
- [x] Environment configuration
- [x] Docker Compose PostgreSQL setup
- [x] Rocket configuration (Rocket.toml)
- [x] Logging infrastructure (tracing)
- [x] HTMX static asset

### Not Implemented (TODO)
- [ ] **Rocket integration in main.rs** - framework not launched
- [ ] **Route handlers** - employees.rs is empty
- [ ] **HTML templates** - base.html is empty
- [ ] **Authentication/JWT** - models exist but logic not implemented
- [ ] **Password hashing** - Argon2/bcrypt not integrated
- [ ] **All endpoint implementations**
- [ ] **Database query functions** - no repository layer
- [ ] **Input validation** at API boundaries
- [ ] **Authorization/RBAC logic**
- [ ] **Tests**

## API Design Guidelines

### Expected Endpoints

**Authentication**
- `POST /auth/login` - User login
- `POST /auth/logout` - User logout

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
- Use Diesel for all database operations
- Validate input at API boundaries
- Use request guards for authentication/authorization
- All monetary values use `rust_decimal::Decimal`
- All IDs use `uuid::Uuid`
- All timestamps use `chrono::DateTime<Utc>`

## Notes

- Stock is automatically managed via database triggers - do NOT manually update stock when processing sales
- All monetary values use `DECIMAL(10,2)` for precision
- UUIDs are used for all primary keys
- The `schema.rs` file is auto-generated by Diesel - DO NOT edit manually
- Run `diesel print-schema` to regenerate after migration changes
