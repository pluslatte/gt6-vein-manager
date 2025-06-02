use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub is_admin: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub invited_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Invitation {
    pub id: String,
    pub email: Option<String>,
    pub token: String,
    pub invited_by: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub used_by: Option<String>,
    pub created_at: DateTime<Utc>,
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
    pub token: String, // 招待トークン
}

#[derive(Debug, Deserialize)]
pub struct InviteForm {
    pub email: Option<String>,
    pub expires_hours: Option<u32>, // デフォルト8時間
}

// 招待リンクのレスポンス
#[derive(Debug, Serialize)]
pub struct InvitationResponse {
    pub id: String,
    pub email: Option<String>,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub invitation_url: String, // フロントエンド用の完全なURL
}

// ユーザー情報レスポンス（パスワードハッシュ除く）
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: String,
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
