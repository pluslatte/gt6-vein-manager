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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

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
            let is_valid = match verify_password(&creds.password, &user.password_hash) {
                Ok(valid) => valid,
                Err(_) => false,
            };

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
