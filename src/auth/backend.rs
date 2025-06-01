use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::models::User;

// axum-login用のユーザー実装
impl AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

// 認証バックエンド
#[derive(Debug, Clone)]
pub struct AuthBackend {
    db: sqlx::MySqlPool,
}

impl AuthBackend {
    pub fn new(db: sqlx::MySqlPool) -> Self {
        Self { db }
    }

    pub async fn ensure_tables(&self) -> anyhow::Result<()> {
        use sqlx::query;

        // ユーザーテーブルの作成
        query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id VARCHAR(36) PRIMARY KEY,
                username VARCHAR(50) UNIQUE NOT NULL,
                email VARCHAR(255),
                password_hash VARCHAR(255) NOT NULL,
                is_admin BOOLEAN DEFAULT FALSE,
                is_active BOOLEAN DEFAULT TRUE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                invited_by CHAR(36),
                INDEX idx_username (username),
                INDEX idx_email (email),
                FOREIGN KEY (invited_by) REFERENCES users(id) ON DELETE SET NULL
            )
            "#,
        )
        .execute(&self.db)
        .await?;

        query(
            r#"
            CREATE TABLE IF NOT EXISTS invitations (
                id VARCHAR(36) PRIMARY KEY,
                email VARCHAR(255),
                token CHAR(36) UNIQUE NOT NULL,
                invited_by CHAR(36) NULL,
                expires_at TIMESTAMP NOT NULL,
                used_at TIMESTAMP NULL,
                used_by CHAR(36) NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                INDEX idx_token (token),
                INDEX idx_email (email),
                FOREIGN KEY (invited_by) REFERENCES users(id) ON DELETE CASCADE,
                FOREIGN KEY (used_by) REFERENCES users(id) ON DELETE SET NULL
            );
            "#,
        )
        .execute(&self.db)
        .await?;

        query(
            r#"
            CREATE TABLE IF NOT EXISTS user_api_keys (
                id VARCHAR(36) PRIMARY KEY,
                user_id CHAR(36) NOT NULL,
                key_name VARCHAR(100) NOT NULL,
                key_hash VARCHAR(255) NOT NULL,
                key_prefix VARCHAR(8) NOT NULL,
                last_used_at TIMESTAMP NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                revoked_at TIMESTAMP NULL,
                INDEX idx_user_id (user_id),
                INDEX idx_key_hash (key_hash),
                INDEX idx_key_prefix (key_prefix),
                FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn check_users_and_generate_invitation(&self) -> anyhow::Result<()> {
        use crate::auth::queries::AuthQueries;

        // アクティブユーザーが存在するかチェック
        let has_users = AuthQueries::has_any_users(&self.db).await?;

        if !has_users {
            println!("\n=== GT6 Vein Manager 初期セットアップ ===");
            println!("システムにユーザーが登録されていません。");
            println!("管理者用の招待リンクを生成します...\n");

            // 環境変数からサーバーURLを取得（デフォルトはlocalhost:24528）
            let server_port = std::env::var("PORT").unwrap_or_else(|_| "24528".to_string());
            let server_host = std::env::var("HOST").unwrap_or_else(|_| "localhost".to_string());
            let server_protocol = std::env::var("PROTOCOL").unwrap_or_else(|_| "http".to_string());
            let base_url = format!("{}://{}:{}", server_protocol, server_host, server_port);

            // システム初期招待を作成
            match AuthQueries::create_system_invitation(&self.db, None).await {
                Ok(invitation) => {
                    let invitation_url =
                        format!("{}/auth/register?token={}", base_url, invitation.token);

                    println!("╭─────────────────────────────────────────────────────────────╮");
                    println!("│                  🎮 GT6 Vein Manager                        │");
                    println!("├─────────────────────────────────────────────────────────────┤");
                    println!(
                        "│  初回管理者アカウント作成用の招待リンクを生成しました                 │"
                    );
                    println!("│                                                             │");
                    println!("│  招待リンク:                                                  │");
                    println!(
                        "│  {}                                                         │",
                        invitation_url
                    );
                    println!("│                                                             │");
                    println!(
                        "│  この招待リンクは 7日間 有効です。                                │"
                    );
                    println!(
                        "│  管理者アカウントを作成後、他のユーザーを招待できます。               │"
                    );
                    println!("│                                                             │");
                    println!(
                        "│  ⚠️  このリンクは一度だけ使用できます。安全に保管してください          │"
                    );
                    println!("╰─────────────────────────────────────────────────────────────╯");
                    println!();

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
    pub remember_me: Option<bool>,
}

#[async_trait::async_trait]
impl AuthnBackend for AuthBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = sqlx::Error;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        use crate::auth::{queries::AuthQueries, utils::verify_password};

        let user = AuthQueries::get_user_by_username(&self.db, &creds.username).await?;

        if let Some(user) = user {
            let is_valid = verify_password(&creds.password, &user.password_hash).unwrap_or(false);

            if is_valid {
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        use crate::auth::queries::AuthQueries;

        AuthQueries::get_user_by_id(&self.db, *user_id).await
    }
}

// セッション層用の型エイリアス
pub type AuthSession = axum_login::AuthSession<AuthBackend>;
