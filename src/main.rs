use std::env;
use std::sync::Arc;
use std::time::Duration;

use dotenv::dotenv;

use serenity::all::GatewayIntents;
use serenity::prelude::*;
use serenity::{all::Ready, async_trait};

use sqlx::sqlite::{SqlitePool, SqliteConnectOptions};

mod config;
use config::{load_config, write_config, Config};

const DEFAULT_PATH: &str = "./config.toml";

struct Handler {
    config: Config,
    conn: SqlitePool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let ctx = Arc::new(ctx);
        let conf = Arc::new(self.config.clone());
        tokio::spawn(async move {
            loop {
                check(Arc::clone(&ctx), Arc::clone(&conf)).await;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }
}

async fn check(_ctx: Arc<Context>, config: Arc<Config>) {
    println!("Config: {:?}", config);
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD").expect("Expected a token in the environment");

    // Write empty config
    // let conf: config::Config = config::Config::default();
    // write_config(DEFAULT_PATH, &conf).await;

    let path = DEFAULT_PATH;
    let mut config = load_config(path).await;
    config.path = path.into();

    let db_options = SqliteConnectOptions::new()
        .filename(&config.db)
        .create_if_missing(true);

    let db_pool = SqlitePool::connect_with(db_options)
        .await
        .expect("Failed to connect to the SQLite database");

    let intents = GatewayIntents::GUILD_MESSAGES;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { config , conn: db_pool })
        .await
        .expect("Error while creating the client!");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
