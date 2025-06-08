use axum::{
    Router, middleware,
    routing::{get, post},
};
use axum_login::AuthManagerLayerBuilder;
use tower_sessions::{Expiry, SessionManagerLayer, cookie::time::Duration};

use crate::auth::{AuthBackend, DieselSessionStore, SESSION_DURATION_DAYS};
use crate::database::AppState;
use crate::handlers::{
    add_vein_handler, login_handler, login_page, logout_handler, me_handler, register_handler,
    register_page, require_auth, search_veins_handler, serve_css, serve_index,
    vein_confirmation_revoke, vein_confirmation_set, vein_depletion_revoke, vein_depletion_set,
    vein_is_bedrock_revoke, vein_is_bedrock_set, vein_revocation_revoke, vein_revocation_set,
    issue_invitation, issue_invitation_html, require_admin,
};

pub async fn create_app(state: AppState) -> anyhow::Result<Router> {
    // セッションストアの初期化
    let session_store = DieselSessionStore::new(state.diesel_pool.clone());

    // セッション管理の設定（7日間有効）
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(SESSION_DURATION_DAYS)));

    let auth_backend = AuthBackend::new(state.diesel_pool.clone());
    auth_backend.check_users_and_generate_invitation().await?;
    
    // 認証バックエンドの設定
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer.clone()).build();

    let app = Router::new()
        .nest(
            "/auth",
            Router::new()
                .route("/login", get(login_page))
                .route("/login", post(login_handler))
                .route("/logout", post(logout_handler))
                .route("/register", get(register_page))
                .route("/register", post(register_handler))
                .route(
                    "/issue-invitation",
                    post(issue_invitation).layer(middleware::from_fn(require_admin)),
                )
                .route("/issue-invitation", get(issue_invitation_html)),
        )
        .nest(
            "/api",
            Router::new()
                .route("/auth/me", get(me_handler))
                .route(
                    "/veins/{vein_id}/confirmation/set",
                    post(vein_confirmation_set),
                )
                .route(
                    "/veins/{vein_id}/confirmation/revoke",
                    post(vein_confirmation_revoke),
                )
                .route("/veins/{vein_id}/depletion/set", post(vein_depletion_set))
                .route(
                    "/veins/{vein_id}/depletion/revoke",
                    post(vein_depletion_revoke),
                )
                .route("/veins/{vein_id}/revocation/set", post(vein_revocation_set))
                .route(
                    "/veins/{vein_id}/revocation/revoke",
                    post(vein_revocation_revoke),
                )
                .route("/veins/{vein_id}/is_bedrock/set", post(vein_is_bedrock_set))
                .route(
                    "/veins/{vein_id}/is_bedrock/revoke",
                    post(vein_is_bedrock_revoke),
                )
                .route("/veins/add", post(add_vein_handler))
                .layer(middleware::from_fn(require_auth)),
        )
        .route(
            "/search",
            get(search_veins_handler).layer(middleware::from_fn(require_auth)),
        )
        .route(
            "/",
            get(serve_index).layer(middleware::from_fn(require_auth)),
        )
        .route(
            "/index.html",
            get(serve_index).layer(middleware::from_fn(require_auth)),
        )
        .route("/styles.css", get(serve_css))
        .layer(auth_layer)
        .layer(session_layer)
        .with_state(state);

    Ok(app)
}
