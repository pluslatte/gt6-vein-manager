use chrono::{Duration, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::auth::utils::{INVITATION_DURATION_HOURS, hash_password};
use crate::models::{ApiKey, Invitation, User};

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
            invited_by: Uuid::nil().to_string(), // システム招待を示すためnil UUID
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
            invited_by: invited_by.to_string(),
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

    /// ユーザーのAPI keyを取得
    pub async fn get_user_api_keys(
        pool: &MySqlPool,
        user_id: &str,
    ) -> Result<Vec<ApiKey>, sqlx::Error> {
        let keys = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM user_api_keys WHERE user_id = ? ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await?;

        Ok(keys)
    }

    /// API keyで認証
    pub async fn authenticate_api_key(
        pool: &MySqlPool,
        key_hash: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT u.* FROM users u
            INNER JOIN user_api_keys ak ON u.id = ak.user_id
            WHERE ak.key_hash = ? AND ak.revoked_at IS NULL AND u.is_active = TRUE
            "#,
        )
        .bind(key_hash)
        .fetch_optional(pool)
        .await?;

        if user.is_some() {
            // 最終使用時刻を更新
            let now = Utc::now();
            sqlx::query("UPDATE user_api_keys SET last_used_at = ? WHERE key_hash = ?")
                .bind(now)
                .bind(key_hash)
                .execute(pool)
                .await?;
        }

        Ok(user)
    }

    /// API keyを作成
    pub async fn create_api_key(
        pool: &MySqlPool,
        user_id: &str,
        key_name: &str,
        key_hash: &str,
        key_prefix: &str,
    ) -> Result<ApiKey, sqlx::Error> {
        let key_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO user_api_keys (id, user_id, key_name, key_hash, key_prefix, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(key_id.to_string())
        .bind(user_id.to_string())
        .bind(key_name)
        .bind(key_hash)
        .bind(key_prefix)
        .bind(now)
        .execute(pool)
        .await?;

        let api_key = ApiKey {
            id: key_id,
            user_id: user_id.to_string(),
            key_name: key_name.to_string(),
            key_hash: key_hash.to_string(),
            key_prefix: key_prefix.to_string(),
            last_used_at: None,
            created_at: now,
            revoked_at: None,
        };

        Ok(api_key)
    }

    /// API keyを無効化
    pub async fn revoke_api_key(
        pool: &MySqlPool,
        key_id: &str,
        user_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let now = Utc::now();

        let result = sqlx::query(
            "UPDATE user_api_keys SET revoked_at = ? WHERE id = ? AND user_id = ? AND revoked_at IS NULL"
        )
        .bind(now)
        .bind(key_id)
        .bind(user_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
