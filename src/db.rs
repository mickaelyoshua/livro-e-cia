use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool, PoolError, PooledConnection},
};
use tracing::{error, info};

pub type DbPool = Pool<ConnectionManager<PgConnection>>;
pub type DbConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn init_pool(database_url: &str) -> Result<DbPool, PoolError> {
    info!("Initializing database connection pool");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(10)
        .build(manager)
        .inspect(|_| info!("Database pool initialized successfully"))
        .inspect_err(|e| error!(error = %e, "Failed to initialize database pool"))
}
