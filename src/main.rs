mod auth;
mod database;
mod handlers;
mod models;

use crate::auth::DieselSessionStore;
use axum::{
    Router,
    routing::{get, post},
};
use axum_login::AuthManagerLayerBuilder;
use tower_sessions::{Expiry, SessionManagerLayer, cookie::time::Duration};

use database::AppState;
use handlers::{
    add_vein_handler, login_handler, login_page, logout_handler, me_handler, register_handler,
    register_page, search_veins_handler, serve_css, serve_index, vein_confirmation_revoke,
    vein_confirmation_set, vein_depletion_revoke, vein_depletion_set, vein_revocation_revoke,
    vein_revocation_set,
};

use crate::{
    auth::{AuthBackend, SESSION_DURATION_DAYS},
    database::create_diesel_pool,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let port = std::env::var("PORT").unwrap_or_else(|_| {
        println!("PORT environment variable not set, using default port 24528");
        "24528".to_string()
    });
    let addr = format!("0.0.0.0:{}", port);

    let diesel_pool = create_diesel_pool().await?;
    let state = AppState {
        diesel_pool: diesel_pool.clone(),
    };

    // セッションストアの初期化
    let session_store = DieselSessionStore::new(diesel_pool.clone());

    // セッション管理の設定（7日間有効）
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(SESSION_DURATION_DAYS)));

    let auth_backend = AuthBackend::new(state.diesel_pool.clone());
    auth_backend.check_users_and_generate_invitation().await?;
    // 認証バックエンドの設定
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer.clone()).build();

    let app = Router::new()
        // 認証関連ルート（認証不要）
        .route("/auth/login", get(login_page))
        .route("/auth/login", post(login_handler))
        .route("/auth/logout", post(logout_handler))
        .route("/auth/register", get(register_page))
        .route("/auth/register", post(register_handler))
        .route("/api/auth/me", get(me_handler))
        // Vein API（認証必要）
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
        .route("/search", get(search_veins_handler))
        .route("/api/veins/add", post(add_vein_handler))
        // 静的ファイル（認証不要）
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .route("/styles.css", get(serve_css))
        .layer(auth_layer)
        .layer(session_layer)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Server running on http://{}", addr);

    axum::serve(listener, app).await?;

    anyhow::Ok(())
}
