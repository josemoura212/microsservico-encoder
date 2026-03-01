use sqlx::{Pool, Postgres, Sqlite, sqlite::SqlitePoolOptions};

macro_rules! impl_with_db {
    (impl<$DB:ident> $Trait:ident<$($TArg:ty),*> for $Type:ident<$($TypeArg:tt),*> { $($body:tt)* }) => {
        impl<$DB> $Trait<$($TArg),*> for $Type<$($TypeArg),*>
        where
            $DB: sqlx::Database,
            for<'q> uuid::Uuid: sqlx::Encode<'q, $DB> + sqlx::Type<$DB> + sqlx::Decode<'q, $DB>,
            for<'q> String: sqlx::Encode<'q, $DB> + sqlx::Type<$DB> + sqlx::Decode<'q, $DB>,
            for<'q> Option<String>: sqlx::Encode<'q, $DB> + sqlx::Type<$DB> + sqlx::Decode<'q, $DB>,
            for<'q> chrono::DateTime<chrono::Utc>:
                sqlx::Encode<'q, $DB> + sqlx::Type<$DB> + sqlx::Decode<'q, $DB>,
            for<'q> &'q uuid::Uuid: sqlx::Encode<'q, $DB>,
            for<'q> &'q String: sqlx::Encode<'q, $DB>,
            for<'q> &'q Option<String>: sqlx::Encode<'q, $DB>,
            for<'q> &'q chrono::DateTime<chrono::Utc>: sqlx::Encode<'q, $DB>,
            for<'q> <$DB as sqlx::Database>::Arguments<'q>: sqlx::IntoArguments<'q, $DB>,
            for<'c> &'c mut <$DB as sqlx::Database>::Connection: sqlx::Executor<'c, Database = $DB>,
            usize: sqlx::ColumnIndex<$DB::Row>,
        {
            $($body)*
        }
    };
    (impl<$DB:ident> $Type:ident<$($TypeArg:tt),*> { $($body:tt)* }) => {
        impl<$DB> $Type<$($TypeArg),*>
        where
            $DB: sqlx::Database,
            for<'q> uuid::Uuid: sqlx::Encode<'q, $DB> + sqlx::Type<$DB> + sqlx::Decode<'q, $DB>,
            for<'q> String: sqlx::Encode<'q, $DB> + sqlx::Type<$DB> + sqlx::Decode<'q, $DB>,
            for<'q> Option<String>: sqlx::Encode<'q, $DB> + sqlx::Type<$DB> + sqlx::Decode<'q, $DB>,
            for<'q> chrono::DateTime<chrono::Utc>:
                sqlx::Encode<'q, $DB> + sqlx::Type<$DB> + sqlx::Decode<'q, $DB>,
            for<'q> &'q uuid::Uuid: sqlx::Encode<'q, $DB>,
            for<'q> &'q String: sqlx::Encode<'q, $DB>,
            for<'q> &'q Option<String>: sqlx::Encode<'q, $DB>,
            for<'q> &'q chrono::DateTime<chrono::Utc>: sqlx::Encode<'q, $DB>,
            for<'q> <$DB as sqlx::Database>::Arguments<'q>: sqlx::IntoArguments<'q, $DB>,
            for<'c> &'c mut <$DB as sqlx::Database>::Connection: sqlx::Executor<'c, Database = $DB>,
            usize: sqlx::ColumnIndex<$DB::Row>,
        {
            $($body)*
        }
    };
}

pub(crate) use impl_with_db;

pub struct Database<T>
where
    T: sqlx::Database,
{
    pub conn: Pool<T>,
}

impl<T: sqlx::Database> Clone for Database<T> {
    fn clone(&self) -> Self {
        Self {
            conn: self.conn.clone(),
        }
    }
}

impl Database<Postgres> {
    pub async fn new(uri: String, auto_migrate: Option<bool>) -> Result<Self, sqlx::Error> {
        let db = Pool::<Postgres>::connect(&uri).await?;

        if auto_migrate.unwrap_or(false) {
            sqlx::migrate!().run(&db).await?;
        }

        Ok(Database { conn: db })
    }
}

impl Database<Sqlite> {
    pub async fn new(uri: String, auto_migrate: Option<bool>) -> Result<Self, sqlx::Error> {
        let db = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&uri)
            .await?;

        if auto_migrate.unwrap_or(false) {
            sqlx::migrate!().run(&db).await?;
        }

        Ok(Database { conn: db })
    }
}
