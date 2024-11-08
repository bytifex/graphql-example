fn setup_sqlx_build_db() {
    use std::fs::{create_dir, remove_dir_all};

    use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .enable_all()
        .build()
        .unwrap();

    runtime.block_on(async move {
        let db_url = "sqlx-build-db/db.sqlite";

        let _ = remove_dir_all("sqlx-build-db");

        if !Sqlite::database_exists(db_url).await.unwrap() {
            let _ = remove_dir_all("sqlx-build-db");
            create_dir("sqlx-build-db").unwrap();

            Sqlite::create_database(db_url).await.unwrap();
        }

        let connection_pool = SqlitePool::connect(db_url).await.unwrap();

        sqlx::migrate!("./db-migrations")
            .run(&connection_pool)
            .await
            .unwrap()
    });
}

fn main() {
    setup_sqlx_build_db();
}
