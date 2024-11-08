use std::{fs::create_dir, path::Path};

use sqlx::{migrate::MigrateDatabase, sqlite::SqliteSynchronous, Sqlite, SqlitePool};

use crate::error::DatabaseOpenError;

#[derive(Clone)]
pub struct Database {
    connection_pool: SqlitePool,
}

impl Database {
    pub async fn open(folder_path: impl AsRef<Path>) -> Result<Self, DatabaseOpenError> {
        let mut db_url = folder_path.as_ref().to_owned();
        db_url.push("db.sqlite");
        let db_url = db_url.to_string_lossy();

        if !Sqlite::database_exists(&db_url).await? {
            let _ = create_dir(&folder_path);
            Sqlite::create_database(&db_url).await?;
        }

        let connection_pool = SqlitePool::connect(&db_url).await?;
        let connection_options = connection_pool.connect_options().as_ref().clone();
        connection_pool
            .set_connect_options(connection_options.synchronous(SqliteSynchronous::Full));

        sqlx::migrate!("./db-migrations")
            .run(&connection_pool)
            .await?;

        Ok(Self { connection_pool })
    }

    pub fn connection_pool_ref(&self) -> &SqlitePool {
        &self.connection_pool
    }
}
