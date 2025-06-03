use anyhow::Result;
use diesel_async::{
    AsyncMysqlConnection,
    pooled_connection::{
        AsyncDieselConnectionManager,
        deadpool::{self},
    },
};

pub type DieselPool = deadpool::Pool<AsyncMysqlConnection>;

#[derive(Clone)]
pub struct AppState {
    pub diesel_pool: DieselPool,
}

pub async fn create_diesel_pool() -> Result<DieselPool> {
    dotenv::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set.");

    let config = AsyncDieselConnectionManager::<AsyncMysqlConnection>::new(database_url);
    let pool = deadpool::Pool::builder(config)
        .max_size(16)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to create diesel pool: {}", e))?;

    println!("Created diesel connection pool");
    Ok(pool)
}
