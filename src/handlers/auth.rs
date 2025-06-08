use std::ops::DerefMut;

use axum::{
    Form,
    extract::{Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Html, Json, Redirect, Response},
};
use serde::Deserialize;

use crate::{
    auth::backend::{AuthSession, Credentials},
    auth::queries::AuthQueries,
    database::connection::AppState,
    models::auth::{InviteForm, LoginForm, RegisterForm, UserResponse},
};

// ログインページ表示
pub async fn login_page() -> Html<&'static str> {
    Html(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>GT6 Vein Manager - ログイン</title>
    <meta charset="utf-8">
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <div class="container">
        <h1>GT6 Vein Manager</h1>
        <h2>ログイン</h2>
        
        <form method="post" action="/auth/login">
            <div class="form-group">
                <label for="username">ユーザー名</label>
                <input type="text" id="username" name="username" required>
            </div>
            
            <div class="form-group">
                <label for="password">パスワード</label>
                <input type="password" id="password" name="password" required>
            </div>
            <button type="submit">ログイン</button>
        </form>
        
        <div class="register-link">
            <p>招待リンクをお持ちですか？ <a href="/auth/register">アカウント作成</a></p>
        </div>
    </div>
</body>
</html>
    "#,
    )
}

// ログイン処理
pub async fn login_handler(
    mut auth_session: AuthSession,
    Form(form): Form<LoginForm>,
) -> Result<Redirect, (StatusCode, String)> {
    let creds = Credentials {
        username: form.username.clone(),
        password: form.password.clone(),
    };

    println!("Login attempt with username: {}", creds.username);

    match auth_session.authenticate(creds).await {
        Ok(Some(user)) => {
            auth_session.login(&user).await.map_err(|e| {
                eprintln!("Login internally failed for user {}: {}", user.username, e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("ログインに失敗しました: {}", e),
                )
            })?;

            println!("User {} logged in successfully", user.username);
            Ok(Redirect::to("/"))
        }
        Ok(None) => {
            println!("Login attempt was invalid for username: {}", form.username);
            Err((
                StatusCode::UNAUTHORIZED,
                "ユーザー名またはパスワードが正しくありません".to_string(),
            ))
        }
        Err(e) => {
            eprintln!("Authentication error for username {}: {}", form.username, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("認証エラー: {}", e),
            ))
        }
    }
}

// ログアウト処理
pub async fn logout_handler(
    mut auth_session: AuthSession,
) -> Result<Redirect, (StatusCode, String)> {
    auth_session.logout().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("ログアウトに失敗しました: {}", e),
        )
    })?;

    Ok(Redirect::to("/auth/login"))
}

// 登録ページ表示
#[derive(Deserialize)]
pub struct RegisterQuery {
    token: Option<String>,
}

pub async fn register_page(Query(query): Query<RegisterQuery>) -> Html<String> {
    let token_input = if let Some(token) = query.token {
        format!(r#"<input type="hidden" name="token" value="{}">"#, token)
    } else {
        r#"<div class="form-group">
            <label for="token">招待トークン</label>
            <input type="text" id="token" name="token" required placeholder="招待リンクから取得したトークン">
        </div>"#.to_string()
    };

    let html = format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>GT6 Vein Manager - アカウント作成</title>
    <meta charset="utf-8">
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <div class="container">
        <h1>GT6 Vein Manager</h1>
        <h2>アカウント作成</h2>
        
        <form method="post" action="/auth/register">
            {token_input}
            
            <div class="form-group">
                <label for="username">ユーザー名</label>
                <input type="text" id="username" name="username" required>
                <div class="password-rules">3-50文字、英数字・アンダースコア・ハイフンのみ</div>
            </div>
            
            <div class="form-group">
                <label for="email">メールアドレス（任意）</label>
                <input type="email" id="email" name="email">
            </div>
            
            <div class="form-group">
                <label for="password">パスワード</label>
                <input type="password" id="password" name="password" required>
                <div class="password-rules">8文字以上</div>
            </div>
            
            <button type="submit">アカウント作成</button>
        </form>
        
        <div class="login-link">
            <p>すでにアカウントをお持ちですか？ <a href="/auth/login">ログイン</a></p>
        </div>
    </div>
</body>
</html>
    "#,
        token_input = token_input
    );

    Html(html)
}

// ユーザー登録処理
pub async fn register_handler(
    State(state): State<AppState>,
    Form(form): Form<RegisterForm>,
) -> Result<Redirect, (StatusCode, String)> {
    use crate::auth::utils::{validate_password, validate_username};

    println!(
        "Attempting to register user with username: {}",
        &form.username
    );

    // データベース接続の取得
    let mut connection = state.diesel_pool.get().await.map_err(|e| {
        eprintln!("Failed to get database connection: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("データベース接続エラー: {}", e),
        )
    })?;
    let connection = connection.deref_mut();

    // バリデーション
    if let Err(e) = validate_username(&form.username) {
        eprintln!("Username validation failed: {}", e);
        return Err((StatusCode::BAD_REQUEST, e));
    }

    if let Err(e) = validate_password(&form.password) {
        eprintln!("Password validation failed: {}", e);
        return Err((StatusCode::BAD_REQUEST, e));
    }

    // 招待トークンの検証
    let invitation = AuthQueries::get_invitation_by_token(connection, &form.token)
        .await
        .map_err(|e| {
            eprintln!("Database error while fetching invitation: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("データベースエラー: {}", e),
            )
        })?;

    let invitation = invitation.ok_or((
        StatusCode::BAD_REQUEST,
        "無効な招待トークンまたは期限切れです".to_string(),
    ))?;

    // ユーザー名の重複チェック
    let existing_user = AuthQueries::get_user_by_username(connection, &form.username)
        .await
        .map_err(|e| {
            eprintln!("Database error while checking existing user: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("データベースエラー: {}", e),
            )
        })?;

    if existing_user.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            "このユーザー名は既に使用されています".to_string(),
        ));
    }

    // ユーザー作成
    // システム招待（invited_by が None）の場合は管理者権限を付与
    let is_admin = invitation.invited_by.is_none();
    let invited_by = invitation.invited_by.as_deref();

    let user = AuthQueries::create_user(
        connection,
        &form.username,
        form.email.as_deref(),
        &form.password,
        invited_by,
        is_admin,
    )
    .await
    .map_err(|e| {
        eprintln!("Error creating user: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("ユーザー作成エラー: {}", e),
        )
    })?;

    // 招待を使用済みにマーク
    AuthQueries::mark_invitation_used(connection, &form.token, &user.id)
        .await
        .map_err(|e| {
            eprintln!("Error marking invitation as used: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("データベースエラー: {}", e),
            )
        })?;

    println!(
        "User registered successfully: {}, with id: {}",
        user.username, user.id
    );
    Ok(Redirect::to("/auth/login"))
}

// 現在のユーザー情報取得（API用）
pub async fn me_handler(auth_session: AuthSession) -> Result<Json<UserResponse>, StatusCode> {
    match auth_session.user {
        Some(user) => Ok(Json(user.into())),
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn issue_invitation(
    State(state): State<AppState>,
    Form(form): Form<InviteForm>,
) -> Result<Html<String>, (StatusCode, String)> {
    // データベース接続の取得
    let mut connection = state.diesel_pool.get().await.map_err(|e| {
        eprintln!("Failed to get database connection: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("データベース接続エラー: {}", e),
        )
    })?;
    let connection = connection.deref_mut();

    // 環境変数からサーバーURLを取得（デフォルトはlocalhost:24528）
    let server_port = std::env::var("PORT").unwrap_or_else(|_| {
        eprintln!("PORT environment variable not set, using default port 24528");
        "24528".to_string()
    });
    let server_host = std::env::var("HOST").unwrap_or_else(|_| {
        eprintln!("HOST environment variable not set, using default localhost");
        "localhost".to_string()
    });
    let server_protocol = std::env::var("PROTOCOL").unwrap_or_else(|_| {
        eprintln!("PROTOCOL environment variable not set, using default http");
        "http".to_string()
    });
    let base_url = format!("{}://{}:{}", server_protocol, server_host, server_port);

    // 招待の保存
    let invitation =
        AuthQueries::create_invitation(connection, form.email.as_deref(), form.email.as_deref())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 招待リンクの生成
    let invitation_url = format!("{}/auth/register?token={}", base_url, invitation.token);

    Ok(Html(format!(
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>GT6 Vein Manager - アカウント作成</title>
    <meta charset="utf-8">
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <div class="container">
        <p>招待リンクが生成されました: <a href="{}">{}</a></p>
        <p>このリンクを招待したユーザーに送信してください。</p>
        <p>リンクの有効期限は8時間です。</p>
        <div class="nav-links">
            <a href="/">ホーム</a>
        </div>
    </div>
</body>
</html>
"#,
        invitation_url, invitation_url
    )))
}

// 認証確認用ミドルウェア
pub async fn require_auth(
    auth_session: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, Redirect> {
    match auth_session.user {
        Some(_) => Ok(next.run(request).await),
        None => Err(Redirect::to("/auth/login")),
    }
}

// 管理者権限確認用ミドルウェア
pub async fn require_admin(
    auth_session: AuthSession,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    match auth_session.user {
        Some(user) if user.is_admin.unwrap_or(false) => Ok(next.run(request).await),
        Some(_) => Err((StatusCode::FORBIDDEN, "管理者権限が必要です".to_string())),
        None => Err((StatusCode::UNAUTHORIZED, "ログインが必要です".to_string())),
    }
}
