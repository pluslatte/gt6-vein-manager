use axum::{Json, Router, extract::State, http::StatusCode, response::Html, routing::get};
use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, prelude::FromRow};

#[derive(Clone)]
struct AppState {
    db_pool: sqlx::Pool<sqlx::MySql>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct Vein {
    id: String,
    name: String,
    x_coord: i32,
    y_coord: Option<i32>,
    z_coord: i32,
    notes: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://testuser:testpassword@localhost:3306/testdb".to_string());

    let pool = MySqlPool::connect(&database_url).await?;
    println!("Connected to the database at {}", database_url);

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
    .execute(&pool)
    .await?;

    let state = AppState { db_pool: pool };

    let app = Router::new()
        .route("/api/veins", get(get_veins_all))
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:24528").await?;
    println!("Server running on http://localhost:24528");

    axum::serve(listener, app).await?;

    anyhow::Ok(())
}

async fn get_veins_all(State(state): State<AppState>) -> Result<Json<Vec<Vein>>, StatusCode> {
    let veins = sqlx::query_as::<_, Vein>(
        r#"
        SELECT id, name, x_coord, y_coord, z_coord, notes
        FROM veins
        ORDER BY created_at DESC
    "#,
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if veins.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(veins))
}

async fn serve_index() -> Html<String> {
    match tokio::fs::read_to_string("public/index.html").await {
        Ok(content) => Html(content),
        Err(_) => Html("<h1>Error: index.html not found</h1>".to_string()),
    }
}
