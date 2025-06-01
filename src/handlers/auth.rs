use axum::{
    Form,
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Json, Redirect},
};
use serde::Deserialize;

use crate::{
    auth::{AuthQueries, AuthSession, Credentials},
    database::AppState,
    models::{LoginForm, RegisterForm, UserResponse},
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
            
            <div class="form-group">
                <div class="checkbox-group">
                    <input type="checkbox" id="remember_me" name="remember_me" value="true">
                    <label for="remember_me">ログイン状態を保持する (7日間)</label>
                </div>
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
        remember_me: form.remember_me,
    };

    match auth_session.authenticate(creds).await {
        Ok(Some(user)) => {
            // Remember me が有効な場合、セッションの有効期限を延長
            if form.remember_me.unwrap_or(false) {
                // セッション期間を7日間に設定
                // これはmain.rsでセッション設定時に処理する
            }

            auth_session.login(&user).await.map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("ログインに失敗しました: {}", e),
                )
            })?;

            Ok(Redirect::to("/"))
        }
        Ok(None) => Err((
            StatusCode::UNAUTHORIZED,
            "ユーザー名またはパスワードが正しくありません".to_string(),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("認証エラー: {}", e),
        )),
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

    println!("Registering user attempt with username: {}", &form.username);
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
    let invitation = AuthQueries::get_invitation_by_token(&state.db_pool, &form.token)
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
    let existing_user = AuthQueries::get_user_by_username(&state.db_pool, &form.username)
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
        &state.db_pool,
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
    AuthQueries::mark_invitation_used(&state.db_pool, &form.token, &user.id)
        .await
        .map_err(|e| {
            eprintln!("Error marking invitation as used: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("データベースエラー: {}", e),
            )
        })?;

    Ok(Redirect::to("/auth/login"))
}

// 現在のユーザー情報取得（API用）
pub async fn me_handler(auth_session: AuthSession) -> Result<Json<UserResponse>, StatusCode> {
    match auth_session.user {
        Some(user) => Ok(Json(user.into())),
        None => Err(StatusCode::UNAUTHORIZED),
    }
}

// 認証確認用ミドルウェア
pub async fn require_auth(auth_session: AuthSession) -> Result<(), Redirect> {
    match auth_session.user {
        Some(_) => Ok(()),
        None => Err(Redirect::to("/auth/login")),
    }
}

// 管理者権限確認用ミドルウェア
pub async fn require_admin(auth_session: AuthSession) -> Result<(), (StatusCode, String)> {
    match auth_session.user {
        Some(user) if user.is_admin => Ok(()),
        Some(_) => Err((StatusCode::FORBIDDEN, "管理者権限が必要です".to_string())),
        None => Err((StatusCode::UNAUTHORIZED, "ログインが必要です".to_string())),
    }
}
