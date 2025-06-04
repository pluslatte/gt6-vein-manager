use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Selectable, Clone, Debug)]
#[diesel(table_name = gt6_vein_manager::schema::user)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub password_hash: String,
    pub is_admin: Option<bool>,
    pub is_active: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
    pub invited_by: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = gt6_vein_manager::schema::invitation)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Invitation {
    pub id: String,
    pub email: Option<String>,
    pub token: String,
    pub invited_by: Option<String>,
    pub expires_at: NaiveDateTime,
    pub used_at: Option<NaiveDateTime>,
    pub used_by: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}

// フォーム用構造体
#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
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
            is_admin: user.is_admin.unwrap_or(false),
            created_at: user
                .created_at
                .unwrap_or(NaiveDateTime::default())
                .and_utc(),
        }
    }
}
