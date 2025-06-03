use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = gt6_vein_manager::schema::vein)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Vein {
    pub id: String,
    pub name: String,
    pub x_coord: i32,
    pub y_coord: Option<i32>,
    pub z_coord: i32,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Selectable)]
#[diesel(belongs_to(Vein))]
#[diesel(table_name = gt6_vein_manager::schema::vein_confirmation)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct VeinConfirmation {
    pub id: String,
    pub vein_id: String,
    pub confirmed: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Selectable)]
#[diesel(belongs_to(Vein))]
#[diesel(table_name = gt6_vein_manager::schema::vein_depletion)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct VeinDepletion {
    pub id: String,
    pub vein_id: String,
    pub depleted: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Selectable)]
#[diesel(belongs_to(Vein))]
#[diesel(table_name = gt6_vein_manager::schema::vein_revocation)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct VeinRevocation {
    pub id: String,
    pub vein_id: String,
    pub revoked: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Selectable)]
#[diesel(belongs_to(Vein))]
#[diesel(table_name = gt6_vein_manager::schema::vein_is_bedrock)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct VeinIsBedrock {
    pub id: String,
    pub vein_id: String,
    pub is_bedrock: Option<bool>,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Selectable)]
#[diesel(belongs_to(Vein))]
#[diesel(table_name = gt6_vein_manager::schema::vein_note)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct VeinNote {
    pub id: String,
    pub vein_id: String,
    pub note: Option<String>,
    pub created_at: Option<NaiveDateTime>,
}
