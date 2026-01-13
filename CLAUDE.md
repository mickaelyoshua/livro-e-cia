# CLAUDE.md - Livro e Cia

## Project Overview

Livro e Cia is a web application for internal management of a Christian bookstore. It handles **stock management**, **sales transactions**, and **statistical analysis of sales data**.

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

**Why NOT JSON API:**
- No need for multiple frontends (mobile, third-party)
- Separate frontend adds complexity without benefit for internal tool
- JSON parsing overhead is negligible, but HTMX is simpler

**Why NOT Leptos/Yew (WebAssembly):**
- SSR + hydration adds significant complexity
- Leptos is 0.7.x - not production-ready for business-critical tools
- WASM build process is complex
- Overkill for internal CRUD + dashboard

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

- **Language:** Rust (Edition 2024)
- **Web Framework:** Rocket
- **Templates:** Tera (via rocket_dyn_templates)
- **Frontend:** HTMX + Chart.js (for statistics)
- **Database:** PostgreSQL 16
- **ORM:** Diesel
- **Authentication:** JWT (stored in HTTP-only cookies)
- **Containerization:** Docker / Docker Compose

## Project Structure

```
livro-e-cia/
├── src/
│   ├── main.rs              # Application entry point
│   └── schema.rs            # Diesel-generated schema (auto-generated)
├── migrations/              # Diesel database migrations
├── Cargo.toml               # Rust dependencies
├── Rocket.toml              # Rocket server configuration
├── diesel.toml              # Diesel ORM configuration
├── docker-compose.yml       # PostgreSQL container setup
├── Makefile                 # Development commands
└── .env                     # Environment variables
```

## Database Schema

### Tables

| Table | Purpose |
|-------|---------|
| `roles` | User roles (admin, manager, employee) |
| `users` | User accounts with authentication |
| `categories` | Book categories (Bíblias, Teologia, etc.) |
| `products` | Book inventory with stock tracking |
| `sales` | Sales transactions |
| `sale_items` | Individual items in each sale (composite PK: sale_id + product_id) |

### Key Features

- **Automatic stock management:** Trigger `decrease_stock_on_sale` decrements product stock when sale items are inserted
- **Timestamp tracking:** Auto-updated `updated_at` columns on users and products
- **Payment methods:** Enum type supporting cash, credit_card, debit_card, pix

## Development Commands

```bash
# Docker
make up              # Start PostgreSQL container
make down            # Stop containers
make logs            # View container logs
make db-shell        # Connect to PostgreSQL shell

# Database
diesel migration run           # Run pending migrations
diesel migration revert        # Revert last migration
diesel migration generate NAME # Create new migration

# Build
cargo build          # Build the project
cargo run            # Run the server
cargo test           # Run tests
```

## Environment Variables

Required in `.env`:

```
DATABASE_URL_DEV=postgresql://user:pass@localhost:5432/livro_cia_db
JWT_SECRET=<256-bit-hex-secret>
ROCKET_PORT=8000
ROCKET_ADDRESS=127.0.0.1
```

## API Design Guidelines

When implementing endpoints, follow this structure:

### Expected Endpoints

**Authentication**
- `POST /auth/login` - User login
- `POST /auth/logout` - User logout

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
- `POST /sales` - Create new sale (auto-decrements stock)
- `GET /sales` - List sales (with date filtering)
- `GET /sales/{id}` - Get sale details with items

**Statistics**
- `GET /stats/sales` - Sales statistics (daily, weekly, monthly)
- `GET /stats/products` - Top selling products
- `GET /stats/categories` - Sales by category

**Stock**
- `GET /stock/low` - Products with low stock
- `PUT /stock/{product_id}` - Adjust stock manually (manager+)

## Coding Conventions

- Use `snake_case` for function and variable names
- Use `PascalCase` for struct and enum names
- Handle errors with `Result<T, E>` types
- Use Diesel for all database operations
- Validate input at API boundaries
- Use request guards for authentication/authorization

## Role-Based Access

| Role | Permissions |
|------|-------------|
| admin | Full access, user management, system configuration |
| manager | Product CRUD, stock management, view all sales |
| employee | Create sales, view own sales, view products |

## Testing

Run tests with:
```bash
cargo test
```

For integration tests, ensure the test database is configured and migrations are run.

## Notes

- Stock is automatically managed via database triggers - do not manually update stock when processing sales
- All monetary values use `DECIMAL(10,2)` for precision
- UUIDs are used for all primary keys
- The application is designed for a Brazilian market (Portuguese categories, PIX payment method)
