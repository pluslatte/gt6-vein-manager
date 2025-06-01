use anyhow::Result;
use bcrypt::{hash, verify, DEFAULT_COST};
use rand::Rng;
use uuid::Uuid;

// パスワード関連の定数
pub const MIN_PASSWORD_LENGTH: usize = 8;
pub const SESSION_DURATION_DAYS: i64 = 7;
pub const INVITATION_DURATION_HOURS: i64 = 8;

// API Key設定
pub const API_KEY_PREFIX: &str = "gt6veinmanager";
pub const API_KEY_LENGTH: usize = 32; // プレフィックス除く

/// パスワードをハッシュ化
pub fn hash_password(password: &str) -> Result<String> {
    let hashed = hash(password, DEFAULT_COST)?;
    Ok(hashed)
}

/// パスワードを検証
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let is_valid = verify(password, hash)?;
    Ok(is_valid)
}

/// パスワードバリデーション
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(format!("パスワードは{}文字以上である必要があります", MIN_PASSWORD_LENGTH));
    }
    Ok(())
}

/// API Keyを生成（gt6veinmanager_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx）
pub fn generate_api_key() -> String {
    let mut rng = rand::thread_rng();
    let key_part: String = (0..API_KEY_LENGTH)
        .map(|_| {
            let chars = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
            chars[rng.gen_range(0..chars.len())] as char
        })
        .collect();
    
    format!("{}_{}", API_KEY_PREFIX, key_part)
}

/// API Keyのプレフィックスを取得（表示用）
pub fn get_api_key_prefix(api_key: &str) -> String {
    if api_key.len() > 12 {
        format!("{}****", &api_key[..12])
    } else {
        api_key.to_string()
    }
}

/// ユーザー名のバリデーション
pub fn validate_username(username: &str) -> Result<(), String> {
    if username.len() < 3 {
        return Err("ユーザー名は3文字以上である必要があります".to_string());
    }
    if username.len() > 50 {
        return Err("ユーザー名は50文字以下である必要があります".to_string());
    }
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err("ユーザー名は英数字、アンダースコア、ハイフンのみ使用可能です".to_string());
    }
    Ok(())
}
