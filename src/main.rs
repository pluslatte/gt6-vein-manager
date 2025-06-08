use gt6_vein_manager::{create_app, create_diesel_pool, AppConfig, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 設定の読み込み
    let config = AppConfig::from_env()?;
    
    // データベース接続プールの作成
    let diesel_pool = create_diesel_pool().await?;
    let state = AppState { diesel_pool };
    
    // アプリケーションの作成
    let app = create_app(state).await?;
    
    // サーバーの起動
    let addr = config.server_address();
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Server running on http://{}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
