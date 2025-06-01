use anyhow::Result;
use sqlx::MySqlPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: MySqlPool,
}

pub async fn initialize_database() -> Result<MySqlPool> {
    dotenv::dotenv().ok();

    let mut database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL environment variable is not set.");
    let database_name =
        std::env::var("DATABASE_NAME").expect("DATABASE_NAME environment variable is not set.");

    if database_url.ends_with('/') {
        database_url.pop();
    }
    let pool = MySqlPool::connect(format!("{}/{}", &database_url, &database_name).as_str()).await?;
    println!("Connected to the database at {}", database_url);

    create_tables(&pool).await?;

    Ok(pool)
}

async fn create_tables(pool: &MySqlPool) -> Result<()> {
    // メインの鉱脈テーブル
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS veins (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            x_coord INT NOT NULL,
            y_coord INT DEFAULT NULL,
            z_coord INT NOT NULL,
            notes TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // 確認テーブル
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS vein_confirmations (
            id VARCHAR(36) PRIMARY KEY,
            vein_id VARCHAR(36) NOT NULL,
            confirmed BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (vein_id) REFERENCES veins(id) ON DELETE CASCADE
        )"#,
    )
    .execute(pool)
    .await?;

    // 枯渇テーブル
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS vein_depletions (
            id VARCHAR(36) PRIMARY KEY,
            vein_id VARCHAR(36) NOT NULL,
            depleted BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (vein_id) REFERENCES veins(id) ON DELETE CASCADE
        )"#,
    )
    .execute(pool)
    .await?;

    // 取り消しテーブル
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS vein_revokations (
            id VARCHAR(36) PRIMARY KEY,
            vein_id VARCHAR(36) NOT NULL,
            revoked BOOLEAN DEFAULT FALSE,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (vein_id) REFERENCES veins(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
