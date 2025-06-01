mod database;
mod handlers;
mod models;

use axum::{
    Router,
    routing::{get, post},
};
use database::{AppState, initialize_database};
use handlers::{
    add_vein_handler, search_veins_handler, serve_css, serve_index, vein_confirmation_revoke,
    vein_confirmation_set, vein_depletion_revoke, vein_depletion_set, vein_revocation_revoke,
    vein_revocation_set,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = initialize_database().await?;
    let state = AppState { db_pool: pool };

    let app = Router::new()
        .route(
            "/api/veins/{vein_id}/confirmation/set",
            post(vein_confirmation_set),
        )
        .route(
            "/api/veins/{vein_id}/confirmation/revoke",
            post(vein_confirmation_revoke),
        )
        .route(
            "/api/veins/{vein_id}/depletion/set",
            post(vein_depletion_set),
        )
        .route(
            "/api/veins/{vein_id}/depletion/revoke",
            post(vein_depletion_revoke),
        )
        .route(
            "/api/veins/{vein_id}/revocation/set",
            post(vein_revocation_set),
        )
        .route(
            "/api/veins/{vein_id}/revocation/revoke",
            post(vein_revocation_revoke),
        )
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
