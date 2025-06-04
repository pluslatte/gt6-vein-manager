use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::{AsyncMysqlConnection, RunQueryDsl};
use uuid::Uuid;

use crate::auth::utils::{INVITATION_DURATION_HOURS, hash_password};
use crate::models::{Invitation, User};
use diesel::dsl::count_star;
use gt6_vein_manager::schema::*;

pub struct AuthQueries;

impl AuthQueries {
    /// システムにユーザーが存在するかチェック
    pub async fn has_any_users(
        connection: &mut AsyncMysqlConnection,
    ) -> Result<bool, diesel::result::Error> {
        let count: i64 = user::table
            .filter(user::is_active.eq(true))
            .select(count_star())
            .first::<i64>(connection)
            .await?;

        Ok(count > 0)
    }

    /// 管理者用の初期招待を作成（システム起動時用）
    pub async fn create_system_invitation(
        connection: &mut AsyncMysqlConnection,
    ) -> Result<Invitation, diesel::result::Error> {
        println!("Creating system invitation...");

        let invitation_id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();
        let now = Utc::now().naive_utc();
        let expires_at = now + Duration::hours((24 * INVITATION_DURATION_HOURS).into()); // 7 days

        // Insert the invitation into the database
        diesel::insert_into(invitation::table)
            .values((
                invitation::id.eq(&invitation_id),
                invitation::email.eq(None::<String>), // System invitation, so NULL
                invitation::token.eq(&token),
                invitation::invited_by.eq::<Option<String>>(None), // System invitation, so NULL
                invitation::expires_at.eq(expires_at),
                invitation::created_at.eq(now),
            ))
            .execute(connection)
            .await?;

        println!("System invitation created: {}", invitation_id);
        // Return the created invitation
        Ok(Invitation {
            id: invitation_id,
            email: None,
            token,
            invited_by: None,
            expires_at,
            used_at: None,
            used_by: None,
            created_at: Some(now),
        })
    }

    /// ユーザー名でユーザーを取得
    pub async fn get_user_by_username(
        connection: &mut AsyncMysqlConnection,
        username: &str,
    ) -> Result<Option<User>, diesel::result::Error> {
        let user = user::table
            .filter(user::username.eq(username))
            .filter(user::is_active.eq(true))
            .first::<User>(connection)
            .await
            .map_err(|_| diesel::result::Error::NotFound)
            .optional()?;

        Ok(user)
    }

    /// UUIDでユーザーを取得
    pub async fn get_user_by_id(
        connection: &mut AsyncMysqlConnection,
        user_id: &str,
    ) -> Result<Option<User>, diesel::result::Error> {
        let user = user::table
            .filter(user::id.eq(user_id))
            .filter(user::is_active.eq(true))
            .first::<User>(connection)
            .await
            .map_err(|_| diesel::result::Error::NotFound)
            .optional()?;

        Ok(user)
    }

    /// 新しいユーザーを作成
    pub async fn create_user(
        connection: &mut AsyncMysqlConnection,
        username: &str,
        email: Option<&str>,
        password: &str,
        invited_by: Option<&str>,
        is_admin: bool,
    ) -> Result<User, diesel::result::Error> {
        let user_id = Uuid::new_v4().to_string();
        let password_hash = hash_password(password).expect("パスワードのハッシュ化に失敗しました");
        let now = Utc::now().naive_utc();

        diesel::insert_into(user::table)
            .values((
                user::id.eq(&user_id),
                user::username.eq(username),
                user::email.eq(email),
                user::password_hash.eq(&password_hash),
                user::is_admin.eq(is_admin),
                user::is_active.eq(true),
                user::created_at.eq(now),
                user::invited_by.eq(invited_by),
            ))
            .execute(connection)
            .await?;

        let user = User {
            id: user_id,
            username: username.to_string(),
            email: email.map(|s| s.to_string()),
            password_hash,
            is_admin: Some(is_admin),
            is_active: Some(true),
            created_at: Some(now),
            invited_by: invited_by.map(|s| s.to_string()),
        };

        Ok(user)
    }

    /// 招待を作成
    pub async fn create_invitation(
        connection: &mut AsyncMysqlConnection,
        email: Option<&str>,
        invited_by: Option<&str>,
    ) -> Result<Invitation, diesel::result::Error> {
        println!("Attempting to create invitation");
        let invitation_id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();
        let now = Utc::now().naive_utc();
        let expires_at = now + Duration::hours((24 * INVITATION_DURATION_HOURS).into()); // 7 days
        let invited_by = Some(
            invited_by
                .filter(|s| !s.is_empty())
                .unwrap_or("anonymous")
                .to_string(),
        );

        if let Err(e) = diesel::insert_into(invitation::table)
            .values((
                invitation::id.eq(&invitation_id),
                invitation::email.eq(email),
                invitation::token.eq(&token),
                invitation::invited_by.eq(invited_by.as_deref()),
                invitation::expires_at.eq(expires_at),
                invitation::created_at.eq(now),
            ))
            .execute(connection)
            .await
        {
            eprintln!("Failed to create invitation: {:?}", e);
            return Err(e);
        }

        let invitation = Invitation {
            id: invitation_id,
            email: email.map(|s| s.to_string()),
            token,
            invited_by,
            expires_at,
            used_at: None,
            used_by: None,
            created_at: Some(now),
        };

        println!(
            "Invitation created: {}, expires at: {}",
            invitation.id, invitation.expires_at
        );
        Ok(invitation)
    }

    /// 招待トークンを取得・検証
    pub async fn get_invitation_by_token(
        connection: &mut AsyncMysqlConnection,
        token: &str,
    ) -> Result<Option<Invitation>, diesel::result::Error> {
        let invitation = invitation::table
            .filter(invitation::token.eq(token))
            .filter(invitation::expires_at.gt(Utc::now().naive_utc()))
            .filter(invitation::used_at.is_null())
            .first::<Invitation>(connection)
            .await
            .optional()?;

        Ok(invitation)
    }

    /// 招待を使用済みにマーク
    pub async fn mark_invitation_used(
        connection: &mut AsyncMysqlConnection,
        token: &str,
        used_by: &str,
    ) -> Result<(), diesel::result::Error> {
        let now = Utc::now().naive_utc();

        diesel::update(invitation::table.filter(invitation::token.eq(token)))
            .set((
                invitation::used_at.eq(Some(now)),
                invitation::used_by.eq(Some(used_by.to_string())),
            ))
            .execute(connection)
            .await?;

        Ok(())
    }
}
