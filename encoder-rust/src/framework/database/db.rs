use sqlx::{Pool, Postgres, Sqlite, sqlite::SqlitePoolOptions};

pub struct Database<T>
where
    T: sqlx::Database,
{
    pub conn: Pool<T>,
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
