use dotenv::dotenv;
use serenity::all::GatewayIntents;
use serenity::prelude::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::{env, sync::Arc};
use tokio::fs;

mod api;
mod bot;
mod config;
mod task;
mod worker;

use bot::Bot;
use config::{load_config, write_config, Config};
use worker::Worker;

const DEFAULT_PATH: &str = "./config.toml";

#[tokio::main]
async fn main() {
    // Environment variables
    // Maybe move them into config.toml?
    dotenv().ok();
    let token = env::var("DISCORD").expect("Expected Discord token in the environment");
    let client_id = env::var("CLIENT_ID").expect("Expected cilent id in the environment");
    let client_secret =
        env::var("CLIENT_SECRET").expect("Expected client secret in the environment");

    // Tries to load config from the default path
    // If that fails, it constructs a Default config and
    // tries to write it on to the path
    let path = DEFAULT_PATH;
    let config: Config = match fs::try_exists(path).await {
        Ok(true) => match load_config(path).await {
            Ok(mut conf) => {
                conf.path = path.into();
                conf
            }
            Err(e) => panic!("Failed to load config! Error: {}", e),
        },
        Ok(false) => {
            let conf: config::Config = config::Config::default();
            if let Err(e) = write_config(&conf, Some(path.into())).await {
                panic!("Failed to write default config! Error: {}", e)
            }
            conf
        }
        Err(e) => panic!("Failed to check config path! Error: {}", e),
    };

    println!("{:#?}", config);

    // Create SQLite database connection
    // Used for storing seen events, etc.
    let db_options = SqliteConnectOptions::new()
        .filename(&config.db)
        .create_if_missing(true);

    let db_pool = SqlitePool::connect_with(db_options)
        .await
        .expect("Failed to connect to the SQLite database");

    let worker = Arc::from(RwLock::from(Worker::new(
        client_id,
        client_secret,
        config.into(),
    )));

    let intents = GatewayIntents::GUILD_MESSAGES;
    let mut client = Client::builder(&token, intents)
        .type_map_insert::<Worker>(worker)
        .event_handler(Bot)
        .await
        .expect("Error while creating the client!");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
