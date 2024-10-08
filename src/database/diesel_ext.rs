use std::fmt::Display;
use std::str::FromStr;

use diesel::deserialize::FromSql;
use diesel::deserialize::FromSqlRow;
use diesel::expression::AsExpression;
use diesel::serialize::ToSql;
use diesel::sql_types::Binary;
use diesel::sqlite::Sqlite;
use diesel::{define_sql_function, QueryResult, SqliteConnection};
use serde::Deserialize;
use serde::Serialize;

#[derive(
    Debug, Clone, Copy, FromSqlRow, AsExpression, Eq, PartialEq, Deserialize, Hash, Serialize,
)]
#[serde(transparent)]
#[diesel(sql_type = Binary)]
pub struct Uuid(pub uuid::Uuid);

impl Uuid {
    pub fn generate() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl FromStr for Uuid {
    type Err = <uuid::Uuid as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <uuid::Uuid as FromStr>::from_str(s).map(Self)
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl ToSql<Binary, Sqlite> for Uuid {
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        <_ as ToSql<Binary, Sqlite>>::to_sql(self.0.as_bytes(), out)
    }
}

impl FromSql<Binary, Sqlite> for Uuid {
    fn from_sql(
        bytes: <Sqlite as diesel::backend::Backend>::RawValue<'_>,
    ) -> diesel::deserialize::Result<Self> {
        let bytes = <Vec<u8> as FromSql<Binary, Sqlite>>::from_sql(bytes)?;
        Ok(Self(uuid::Uuid::from_slice(bytes.as_slice())?))
    }
}

define_sql_function! {
    fn generate_uuid() -> Binary;
}

pub fn register_functions(conn: &mut SqliteConnection) -> QueryResult<()> {
    generate_uuid_utils::register_nondeterministic_impl(conn, Uuid::generate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::Connection;
    use diesel::IntoSql;
    use diesel::RunQueryDsl;

    #[test]
    fn test_to_sql() {
        let mut conn = SqliteConnection::establish(":memory:").unwrap();
        let uuid = Uuid::generate();
        let result = diesel::select(uuid.into_sql::<Binary>())
            .get_result::<Vec<u8>>(&mut conn)
            .unwrap();

        assert_eq!(result, uuid.0.as_bytes());
    }

    #[test]
    fn test_from_sql() {
        let mut conn = SqliteConnection::establish(":memory:").unwrap();
        let uuid = Uuid::generate();
        let result = diesel::select(uuid.0.as_bytes().into_sql::<Binary>())
            .get_result::<Uuid>(&mut conn)
            .unwrap();

        assert_eq!(result, uuid);
    }

    #[test]
    fn test_generate_uuid() {
        let mut conn = SqliteConnection::establish(":memory:").unwrap();

        register_functions(&mut conn).unwrap();

        let res = diesel::select(generate_uuid()).get_result::<Uuid>(&mut conn);
        assert!(res.is_ok());
    }
}
