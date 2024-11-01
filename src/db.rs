use std::fs;
use std::path::PathBuf;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

pub async fn get_db(path: &PathBuf) -> SqlitePool {
    if !path.try_exists().unwrap_or(false) {
        panic!("Database {:?} doesn't exist!", path);
    }

    // Create SQLite database connection
    // Used for storing seen events, news, etc.
    let db_options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);

    let db_pool = SqlitePool::connect_with(db_options)
        .await
        .expect("Failed to connect to the SQLite database");

    println!("Connected to {:?}", fs::canonicalize(path));

    db_pool
}
