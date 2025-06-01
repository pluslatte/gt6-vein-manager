use axum_login::{AuthUser, AuthnBackend, UserId};
use serde::{Deserialize, Serialize};
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
                id CHAR(36) PRIMARY KEY,
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
                id CHAR(36) PRIMARY KEY,
                email VARCHAR(255),
                token CHAR(36) UNIQUE NOT NULL,
                invited_by CHAR(36) NOT NULL,
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
                id CHAR(36) PRIMARY KEY,
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
