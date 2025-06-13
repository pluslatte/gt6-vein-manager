use std::collections::HashMap;

use crate::models::sessions::{NewSession, Session};
use crate::schema::sessions;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tower_sessions::{
    SessionStore,
    session::{Id, Record},
    session_store,
};

use crate::auth::backend::DbPool;

/// A session store implementation using Diesel ORM for database interactions.
#[derive(Clone)]
pub struct DieselSessionStore {
    pool: DbPool,
}

// Implementing Debug for requirements in tower-sessions (maybe)
// I don't know why it's needed and whether it's actually required...
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
        let mut connection = self.pool.get().await.map_err(|e| {
            session_store::Error::Backend(format!("Failed to get connection: {}", e))
        })?;

        let data_str = serialize_session_data(&record.data)?;
        let expiry_naive = offset_to_naive_datetime(record.expiry_date);

        // To avoid duplicate session IDs
        loop {
            let session_id_str = record.id.to_string();

            // To check if the session ID already exists
            let existing_session: Option<Session> = sessions::table
                .filter(sessions::id.eq(&session_id_str))
                .first(&mut connection)
                .await
                .optional()
                .map_err(|e| {
                    session_store::Error::Backend(format!("Failed to check session: {}", e))
                })?;

            if existing_session.is_none() {
                let new_session = NewSession {
                    id: session_id_str,
                    data: data_str,
                    expiry_date: expiry_naive,
                };

                diesel::insert_into(sessions::table)
                    .values(new_session)
                    .execute(&mut connection)
                    .await
                    .map_err(|e| {
                        session_store::Error::Backend(format!("Failed to create session: {}", e))
                    })?;

                return Ok(());
            } else {
                // To avoid duplicate session IDs
                record.id = Id::default();
            }
        }
    }

    async fn save(&self, record: &Record) -> session_store::Result<()> {
        let mut connection = self.pool.get().await.map_err(|e| {
            session_store::Error::Backend(format!("Failed to get connection: {}", e))
        })?;

        let session_id_str = record.id.to_string();
        let data_str = serialize_session_data(&record.data)?;
        let expiry_naive = offset_to_naive_datetime(record.expiry_date);

        // To check if the session already exists
        let existing_session: Option<Session> = sessions::table
            .filter(sessions::id.eq(&session_id_str))
            .first(&mut connection)
            .await
            .optional()
            .map_err(|e| {
                session_store::Error::Backend(format!("Failed to check session: {}", e))
            })?;

        // Need to change operation based on whether the session exists
        if existing_session.is_some() {
            diesel::update(sessions::table.filter(sessions::id.eq(&session_id_str)))
                .set((
                    sessions::data.eq(data_str),
                    sessions::expiry_date.eq(expiry_naive),
                ))
                .execute(&mut connection)
                .await
                .map_err(|e| {
                    session_store::Error::Backend(format!("Failed to update session: {}", e))
                })?;
        } else {
            let new_session = NewSession {
                id: session_id_str,
                data: data_str,
                expiry_date: expiry_naive,
            };

            diesel::insert_into(sessions::table)
                .values(new_session)
                .execute(&mut connection)
                .await
                .map_err(|e| {
                    session_store::Error::Backend(format!("Failed to create session: {}", e))
                })?;
        }

        Ok(())
    }

    async fn load(&self, id: &Id) -> session_store::Result<Option<Record>> {
        let mut connection = self.pool.get().await.map_err(|e| {
            session_store::Error::Backend(format!("Failed to get connection: {}", e))
        })?;

        let session_id_str = id.to_string();

        // Need to check if the session exists with given ID
        let session: Option<Session> = sessions::table
            .filter(sessions::id.eq(&session_id_str))
            .first(&mut connection)
            .await
            .optional()
            .map_err(|e| session_store::Error::Backend(format!("Failed to load session: {}", e)))?;

        match session {
            Some(session) => {
                // Because tower-sessions expects the expiry date in UTC OffsetDateTime
                let expiry_offset = naive_datetime_to_offset(session.expiry_date);
                if expiry_offset < time::OffsetDateTime::now_utc() {
                    self.delete(id).await?;
                    return Ok(None);
                }

                let data = deserialize_session_data(&session.data)?;

                // Session store expects a Record with an ID, data, and expiry date
                Ok(Some(Record {
                    id: *id,
                    data,
                    expiry_date: expiry_offset,
                }))
            }
            // None means the session does not exist
            None => Ok(None),
        }
    }

    async fn delete(&self, id: &Id) -> session_store::Result<()> {
        let mut connection = self.pool.get().await.map_err(|e| {
            session_store::Error::Backend(format!("Failed to get connection: {}", e))
        })?;

        let session_id_str = id.to_string();

        diesel::delete(sessions::table)
            .filter(sessions::id.eq(session_id_str))
            .execute(&mut connection)
            .await
            .map_err(|e| {
                session_store::Error::Backend(format!("Failed to delete session: {}", e))
            })?;

        Ok(())
    }
}

/// Serializes session data from a HashMap into a JSON string.
fn serialize_session_data(
    data: &HashMap<String, serde_json::Value>,
) -> session_store::Result<String> {
    serde_json::to_string(data).map_err(|e| session_store::Error::Backend(e.to_string()))
}

/// Deserializes session data from a JSON string into a HashMap.
fn deserialize_session_data(
    data: &str,
) -> session_store::Result<HashMap<String, serde_json::Value>> {
    serde_json::from_str(data).map_err(|e| session_store::Error::Backend(e.to_string()))
}

/// Converts a `NaiveDateTime` to a `time::OffsetDateTime` in UTC.
fn naive_datetime_to_offset(naive: NaiveDateTime) -> time::OffsetDateTime {
    let timestamp = naive.and_utc().timestamp();
    let nanos = naive.and_utc().timestamp_subsec_nanos();
    time::OffsetDateTime::from_unix_timestamp_nanos(
        (timestamp as i128) * 1_000_000_000 + nanos as i128,
    )
    .unwrap_or(time::OffsetDateTime::UNIX_EPOCH)
}

/// Converts a `time::OffsetDateTime` to a `NaiveDateTime` in UTC.
fn offset_to_naive_datetime(offset: time::OffsetDateTime) -> NaiveDateTime {
    let timestamp = offset.unix_timestamp();
    let nanos = offset.nanosecond();
    chrono::DateTime::from_timestamp(timestamp, nanos)
        .unwrap_or(chrono::DateTime::UNIX_EPOCH)
        .naive_utc()
}
