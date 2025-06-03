use anyhow::Result;
use diesel_async::{
    AsyncConnection, AsyncMysqlConnection,
    pooled_connection::{
        AsyncDieselConnectionManager,
        deadpool::{self, PoolError},
    },
};
use sqlx::MySqlPool;

pub type DieselPool = deadpool::Pool<AsyncMysqlConnection>;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: MySqlPool,
    pub diesel_pool: DieselPool,
}

pub async fn connect_session_store_mysql() -> Result<MySqlPool> {
    dotenv::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set.");

    let pool = MySqlPool::connect(&database_url).await?;
    println!(
        "Connected to the session store database at {}",
        database_url
    );

    Ok(pool)
}

pub async fn connect_diegel_mysql() -> AsyncMysqlConnection {
    dotenv::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set.");
    AsyncMysqlConnection::establish(&database_url)
        .await
        .expect("Error connecting to MySQL database")
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
