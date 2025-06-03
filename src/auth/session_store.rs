use std::collections::HashMap;

use chrono::NaiveDateTime;
use tower_sessions::{
    SessionStore,
    session::{Id, Record},
    session_store,
};

use crate::auth::DbPool;

#[derive(Clone)]
pub struct DieselSessionStore {
    pool: DbPool,
}

impl std::fmt::Debug for DieselSessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DieselSessionStore")
            .field("pool", &"<Database Pool>")
            .finish()
    }
}

impl DieselSessionStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SessionStore for DieselSessionStore {
    async fn create(&self, record: &mut Record) -> session_store::Result<()> {
        todo!("Implement create session logic using Diesel");
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        todo!("Implement save session logic using Diesel");
    }

    async fn load(&self, id: &Id) -> session_store::Result<Option<Record>> {
        todo!("Implement load session logic using Diesel");
    }

    async fn delete(&self, id: &Id) -> session_store::Result<()> {
        todo!("Implement delete session logic using Diesel");
    }
}

fn serialize_session_data(
    data: &HashMap<String, serde_json::Value>,
) -> session_store::Result<String> {
    serde_json::to_string(data).map_err(|e| session_store::Error::Backend(e.to_string()))
}

fn deserialize_session_data(
    data: &str,
) -> session_store::Result<HashMap<String, serde_json::Value>> {
    serde_json::from_str(data).map_err(|e| session_store::Error::Backend(e.to_string()))
}

fn naive_datetime_to_offset(naive: NaiveDateTime) -> time::OffsetDateTime {
    todo!("Convert NaiveDateTime to OffsetDateTime")
}

fn offset_to_naive_datetime(offset: time::OffsetDateTime) -> NaiveDateTime {
    todo!("Convert OffsetDateTime to NaiveDateTime")
}
