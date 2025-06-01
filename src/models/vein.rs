use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Vein {
    pub id: String,
    pub name: String,
    pub x_coord: i32,
    pub y_coord: Option<i32>,
    pub z_coord: i32,
    pub notes: Option<String>,
    pub confirmed: bool,
    pub depleted: bool,
    pub created_at: Option<DateTime<Utc>>,
}

impl Vein {
    pub fn format_y_coord(&self) -> String {
        self.y_coord
            .map_or_else(|| "-".to_string(), |y| y.to_string())
    }

    pub fn format_notes(&self) -> &str {
        self.notes.as_deref().unwrap_or("-")
    }

    pub fn format_created_at(&self) -> String {
        self.created_at.map_or_else(
            || "-".to_string(),
            |dt| dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        )
    }

    pub fn confirmed_symbol(&self) -> &'static str {
        if self.confirmed { "✓" } else { "✗" }
    }

    pub fn depleted_symbol(&self) -> &'static str {
        if self.depleted { "✓" } else { "✗" }
    }
}
