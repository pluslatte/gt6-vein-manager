use chrono::{Duration, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::auth::utils::{INVITATION_DURATION_HOURS, hash_password};
use crate::models::{Invitation, User};

pub struct AuthQueries;

impl AuthQueries {
    /// システムにユーザーが存在するかチェック
    pub async fn has_any_users(pool: &MySqlPool) -> Result<bool, sqlx::Error> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE is_active = TRUE")
            .fetch_one(pool)
            .await?;

        Ok(count.0 > 0)
    }

    /// 管理者用の初期招待を作成（システム起動時用）
    pub async fn create_system_invitation(
        pool: &MySqlPool,
        email: Option<&str>,
    ) -> Result<Invitation, sqlx::Error> {
        let invitation_id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();
        let now = Utc::now();
        // 初期招待は1週間有効
        let expires_at = now + Duration::hours(24 * 7); // 7 days

        // システム招待なので invited_by は NULL
        sqlx::query(
            r#"
            INSERT INTO invitations (id, email, token, invited_by, expires_at, created_at)
            VALUES (?, ?, ?, NULL, ?, ?)
            "#,
        )
        .bind(&invitation_id)
        .bind(email)
        .bind(&token)
        .bind(expires_at)
        .bind(now)
        .execute(pool)
        .await?;

        let invitation = Invitation {
            id: invitation_id,
            email: email.map(|s| s.to_string()),
            token,
            invited_by: None, // システム招待なので None
            expires_at,
            used_at: None,
            used_by: None,
            created_at: now,
        };

        Ok(invitation)
    }

    /// ユーザー名でユーザーを取得
    pub async fn get_user_by_username(
        pool: &MySqlPool,
        username: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE username = ? AND is_active = TRUE",
        )
        .bind(username)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// UUIDでユーザーを取得
    pub async fn get_user_by_id(
        pool: &MySqlPool,
        user_id: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        let user =
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ? AND is_active = TRUE")
                .bind(user_id.to_string())
                .fetch_optional(pool)
                .await?;

        Ok(user)
    }

    /// 新しいユーザーを作成
    pub async fn create_user(
        pool: &MySqlPool,
        username: &str,
        email: Option<&str>,
        password: &str,
        invited_by: Option<&str>,
        is_admin: bool,
    ) -> Result<User, sqlx::Error> {
        let user_id = Uuid::new_v4().to_string();
        let password_hash = hash_password(password).expect("パスワードのハッシュ化に失敗しました");
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, is_admin, is_active, created_at, invited_by)
            VALUES (?, ?, ?, ?, ?, TRUE, ?, ?)
            "#
        )
        .bind(&user_id)
        .bind(username)
        .bind(email)
        .bind(&password_hash)
        .bind(is_admin)
        .bind(now)
        .bind(invited_by)
        .execute(pool)
        .await?;

        let user = User {
            id: user_id,
            username: username.to_string(),
            email: email.map(|s| s.to_string()),
            password_hash,
            is_admin,
            is_active: true,
            created_at: now,
            invited_by: invited_by.map(|s| s.to_string()),
        };

        Ok(user)
    }

    /// 招待を作成
    pub async fn create_invitation(
        pool: &MySqlPool,
        email: Option<&str>,
        invited_by: &str,
    ) -> Result<Invitation, sqlx::Error> {
        let invitation_id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + Duration::hours(INVITATION_DURATION_HOURS);

        sqlx::query(
            r#"
            INSERT INTO invitations (id, email, token, invited_by, expires_at, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&invitation_id)
        .bind(email)
        .bind(&token)
        .bind(invited_by)
        .bind(expires_at)
        .bind(now)
        .execute(pool)
        .await?;

        let invitation = Invitation {
            id: invitation_id,
            email: email.map(|s| s.to_string()),
            token,
            invited_by: Some(invited_by.to_string()),
            expires_at,
            used_at: None,
            used_by: None,
            created_at: now,
        };

        Ok(invitation)
    }

    /// 招待トークンを取得・検証
    pub async fn get_invitation_by_token(
        pool: &MySqlPool,
        token: &str,
    ) -> Result<Option<Invitation>, sqlx::Error> {
        let invitation = sqlx::query_as::<_, Invitation>(
            r#"
            SELECT * FROM invitations 
            WHERE token = ? AND expires_at > NOW() AND used_at IS NULL
            "#,
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;

        Ok(invitation)
    }

    /// 招待を使用済みにマーク
    pub async fn mark_invitation_used(
        pool: &MySqlPool,
        token: &str,
        used_by: &str,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now();

        sqlx::query("UPDATE invitations SET used_at = ?, used_by = ? WHERE token = ?")
            .bind(now)
            .bind(used_by)
            .bind(token)
            .execute(pool)
            .await?;

        Ok(())
    }
}
