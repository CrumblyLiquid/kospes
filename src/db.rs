use std::path::PathBuf;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

pub async fn get_db(path: &PathBuf) -> SqlitePool {
    // Create SQLite database connection
    // Used for storing seen events, etc.
    let db_options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);

    let db_pool = SqlitePool::connect_with(db_options)
        .await
        .expect("Failed to connect to the SQLite database");

    db_pool
}
