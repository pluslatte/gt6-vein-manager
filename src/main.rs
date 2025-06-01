mod models;
mod database;
mod handlers;

use axum::{
    Router,
    routing::{get, post},
};
use database::{initialize_database, AppState};
use handlers::{
    get_veins_all,
    serve_index, serve_css,
    search_veins_handler, add_vein_handler,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = initialize_database().await?;
    let state = AppState { db_pool: pool };

    let app = Router::new()
        .route("/api/veins", get(get_veins_all))
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .route("/styles.css", get(serve_css))
        .route("/search", get(search_veins_handler))
        .route("/add", post(add_vein_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:24528").await?;
    println!("Server running on http://localhost:24528");

    axum::serve(listener, app).await?;

    anyhow::Ok(())
}
