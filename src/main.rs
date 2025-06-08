use gt6_vein_manager::{
    app::create_app, config::AppConfig, database::connection::AppState,
    database::connection::create_diesel_pool,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::from_env()?;

    let diesel_pool = create_diesel_pool().await?;
    let state = AppState { diesel_pool };

    let app = create_app(state).await?;

    let addr = config.server_address();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Server running on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
