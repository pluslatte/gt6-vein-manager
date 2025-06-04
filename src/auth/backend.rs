use axum_login::{AuthUser, AuthnBackend, UserId};
use diesel_async::{AsyncMysqlConnection, pooled_connection::deadpool::Pool};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::models::User;

// カスタムエラー型の定義
#[derive(Debug)]
pub enum AuthError {
    Database(diesel::result::Error),
    Pool(diesel_async::pooled_connection::deadpool::PoolError),
    PasswordHashing(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::Database(e) => write!(f, "Database error: {}", e),
            AuthError::Pool(e) => write!(f, "Connection pool error: {}", e),
            AuthError::PasswordHashing(e) => write!(f, "Password hashing error: {}", e),
        }
    }
}

impl std::error::Error for AuthError {}

// エラー型変換の実装
impl From<diesel::result::Error> for AuthError {
    fn from(error: diesel::result::Error) -> Self {
        AuthError::Database(error)
    }
}

impl From<diesel_async::pooled_connection::deadpool::PoolError> for AuthError {
    fn from(error: diesel_async::pooled_connection::deadpool::PoolError) -> Self {
        AuthError::Pool(error)
    }
}

// axum-login用のユーザー実装
impl AuthUser for User {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

pub type DbPool = Pool<AsyncMysqlConnection>;
#[derive(Clone)]
pub struct AuthBackend {
    pub db: DbPool,
}

impl AuthBackend {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub async fn check_users_and_generate_invitation(&self) -> anyhow::Result<()> {
        use crate::auth::queries::AuthQueries;

        let mut connection = self.db.get().await?;
        // アクティブユーザーが存在するかチェック
        let has_users = AuthQueries::has_any_users(&mut connection).await?;

        if !has_users {
            println!("\n=== GT6 Vein Manager 初期セットアップ ===");
            println!("システムにユーザーが登録されていません。");
            println!("管理者用の招待リンクを生成します...\n");

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

            // システム初期招待を作成
            match AuthQueries::create_system_invitation(&mut connection).await {
                Ok(invitation) => {
                    let invitation_url =
                        format!("{}/auth/register?token={}", base_url, invitation.token);

                    println!("GT6 Vein Manager");
                    println!("初回管理者アカウント作成用の招待リンクを生成しました");
                    println!("{}", invitation_url);
                    println!("この招待リンクは 7日間 有効です。");
                    println!("管理者アカウントを作成後、他のユーザーを招待できます。");
                    println!("⚠️このリンクは一度だけ使用できます。安全に保管してください");
                    // 有効期限も表示
                    println!(
                        "有効期限: {}",
                        invitation.expires_at.format("%Y-%m-%d %H:%M:%S UTC")
                    );
                    println!("招待ID: {}", invitation.id);
                    println!();
                }
                Err(e) => {
                    println!("❌ 招待リンクの生成に失敗しました: {}", e);
                }
            }
        } else {
            println!("✅ ユーザーが既に登録されています。");
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[async_trait::async_trait]
impl AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        use crate::auth::{queries::AuthQueries, utils::verify_password};

        let mut connection = self.db.get().await?;

        let user = AuthQueries::get_user_by_username(&mut connection, &creds.username).await?;

        if let Some(user) = user {
            let is_valid = verify_password(&creds.password, &user.password_hash)
                .map_err(|e| AuthError::PasswordHashing(e.to_string()))?;

            if is_valid {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        use crate::auth::queries::AuthQueries;

        let mut connection = self.db.get().await?;

        Ok(AuthQueries::get_user_by_id(&mut connection, user_id).await?)
    }
}

// セッション層用の型エイリアス
pub type AuthSession = axum_login::AuthSession<AuthBackend>;
