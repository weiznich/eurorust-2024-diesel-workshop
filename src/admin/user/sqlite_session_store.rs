//! Custom `SessionStore` implementation to store session data in our database
//!
//! `tower_sessions` does not provide this out of the box
use crate::database::schema::session_records;
use axum_login::tower_sessions::session::{self, Record};
use axum_login::tower_sessions::session_store;
use axum_login::tower_sessions::SessionStore;
use diesel::prelude::*;
use diesel::sqlite::Sqlite;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct SqliteSessionStore {
    pub(crate) pool: deadpool_diesel::sqlite::Pool,
}

impl std::fmt::Debug for SqliteSessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteSessionStore").finish()
    }
}

impl SqliteSessionStore {
    pub fn new(pool: deadpool_diesel::sqlite::Pool) -> Self {
        Self { pool }
    }

    pub(crate) async fn with_connection<T: Send + 'static>(
        &self,
        c: impl FnOnce(&mut SqliteConnection) -> QueryResult<T> + Send + 'static,
    ) -> session_store::Result<T> {
        self.pool
            .get()
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?
            .interact(c)
            .await
            .map_err(|e| session_store::Error::Backend(e.to_string()))?
            .map_err(|e| session_store::Error::Backend(e.to_string()))
    }
}

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = session_records)]
#[diesel(check_for_backend(Sqlite))]
pub(crate) struct SessionRecord {
    pub(crate) id: Vec<u8>,
    pub(crate) data: String,
    pub(crate) expiry_date: OffsetDateTime,
}

impl TryFrom<&'_ Record> for SessionRecord {
    type Error = serde_json::Error;

    fn try_from(value: &'_ Record) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.0.to_be_bytes().to_vec(),
            data: serde_json::to_string(&value.data)?,
            expiry_date: value.expiry_date,
        })
    }
}

impl TryFrom<SessionRecord> for Record {
    type Error = session_store::Error;

    fn try_from(value: SessionRecord) -> Result<Self, Self::Error> {
        let id = i128::from_be_bytes(
            *<&[u8; 16] as TryFrom<&[u8]>>::try_from(&value.id)
                .map_err(|e| session_store::Error::Decode(e.to_string()))?,
        );
        let data = serde_json::from_str(&value.data)
            .map_err(|e| session_store::Error::Decode(e.to_string()))?;
        Ok(Record {
            id: session::Id(id),
            data,
            expiry_date: value.expiry_date,
        })
    }
}

#[async_trait::async_trait]
impl SessionStore for SqliteSessionStore {
    async fn save(&self, session_record: &Record) -> session_store::Result<()> {
        let record_to_insert = SessionRecord::try_from(session_record)
            .map_err(|e| session_store::Error::Decode(e.to_string()))?;
        self.with_connection(move |conn| {
            diesel::insert_into(session_records::table)
                .values(&record_to_insert)
                .on_conflict(session_records::id)
                .do_update()
                .set((
                    session_records::data.eq(&record_to_insert.data),
                    session_records::expiry_date.eq(&record_to_insert.expiry_date),
                ))
                .execute(conn)
                .map(|_| ())
        })
        .await
    }

    async fn load(&self, session_id: &session::Id) -> session_store::Result<Option<Record>> {
        let id = session_id.0.to_be_bytes();
        let record = self
            .with_connection(move |conn| {
                session_records::table
                    .find(id)
                    .select(SessionRecord::as_select())
                    .first(conn)
                    .optional()
            })
            .await?;
        record.map(TryInto::try_into).transpose()
    }

    async fn delete(&self, session_id: &session::Id) -> session_store::Result<()> {
        let id = session_id.0.to_be_bytes();
        self.with_connection(move |conn| {
            diesel::delete(session_records::table.find(id)).execute(conn)?;
            Ok(())
        })
        .await
    }
}
