use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub is_admin: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub invited_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invitation {
    pub id: Uuid,
    pub email: Option<String>,
    pub token: Uuid,
    pub invited_by: Uuid,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub used_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub key_name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

// API keyの表示用（実際のキーは含まない）
#[derive(Debug, Serialize)]
pub struct ApiKeyDisplay {
    pub id: Uuid,
    pub key_name: String,
    pub key_prefix: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub is_revoked: bool,
}

impl From<ApiKey> for ApiKeyDisplay {
    fn from(key: ApiKey) -> Self {
        Self {
            id: key.id,
            key_name: key.key_name,
            key_prefix: key.key_prefix,
            last_used_at: key.last_used_at,
            created_at: key.created_at,
            is_revoked: key.revoked_at.is_some(),
        }
    }
}

// フォーム用構造体
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub remember_me: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: Option<String>,
    pub password: String,
    pub token: Uuid, // 招待トークン
}

#[derive(Debug, Deserialize)]
pub struct InviteForm {
    pub email: Option<String>,
    pub expires_hours: Option<u32>, // デフォルト8時間
}

#[derive(Debug, Deserialize)]
pub struct CreateApiKeyForm {
    pub key_name: String,
}

// API key生成時のレスポンス（一度だけ表示）
#[derive(Debug, Serialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub key_name: String,
    pub api_key: String, // gt6veinmanager_xxxxxxxxxxxx 形式
    pub created_at: DateTime<Utc>,
}

// 招待リンクのレスポンス
#[derive(Debug, Serialize)]
pub struct InvitationResponse {
    pub id: Uuid,
    pub email: Option<String>,
    pub token: Uuid,
    pub expires_at: DateTime<Utc>,
    pub invitation_url: String, // フロントエンド用の完全なURL
}

// ユーザー情報レスポンス（パスワードハッシュ除く）
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: Option<String>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            is_admin: user.is_admin,
            created_at: user.created_at,
        }
    }
}
