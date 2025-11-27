use diesel::{
    r2d2::{ConnectionManager, Pool, PoolError, PooledConnection},
    PgConnection,
};
use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome},
    Request, State,
};

// Connection Pool Type
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

// Single connection from pool
pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;

// Initialize databa connection pool
pub fn init_pool(database_url: &str) -> Result<DbPool, PoolError> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder().max_size(10).build(manager)
}

// Rocket request guard for database connections
// Can't implement foreign traits on foreign types (orphan rule). Necessary to implement
// FromRequest trait
pub struct DbConnection(pub DbConn);

// Trait to enable async trait (Rust does not have natively)
#[rocket::async_trait]
// When identify the DbConnection needed on the route, it will automatically run this
impl<'r> FromRequest<'r> for DbConnection {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // get the DbPool from managed state
        // guard is a piece of code that runs before the route handler
        let pool = match req.guard::<&State<DbPool>>().await {
            Outcome::Success(pool) => pool,
            Outcome::Error((status, _)) => return Outcome::Error((status, ())),
            // Route does not apply to this request, try another route
            Outcome::Forward(forward) => return Outcome::Forward(forward),
        };

        match pool.get() {
            Ok(conn) => Outcome::Success(DbConnection(conn)),
            Err(_) => {
                log::error!("Failed to get database connection from pool");
                Outcome::Error((Status::ServiceUnavailable, ()))
            }
        }
    }
}
