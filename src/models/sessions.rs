use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = crate::schema::sessions)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Session {
    pub id: String,
    pub data: String,
    pub expiry_date: NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::sessions)]
pub struct NewSession {
    pub id: String,
    pub data: String,
    pub expiry_date: NaiveDateTime,
}
