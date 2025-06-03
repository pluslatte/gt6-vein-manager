use anyhow::Result;
use diesel_async::{AsyncConnection, AsyncMysqlConnection};
use sqlx::MySqlPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: MySqlPool,
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
