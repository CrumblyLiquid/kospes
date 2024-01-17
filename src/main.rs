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

mod task;
use task::Task;

mod sirius;
use sirius::Sirius;

const DEFAULT_PATH: &str = "./config.toml";

struct Handler {
    client_id: String,
    client_secret: String,
    tasks: Vec<Task>,
    conn: SqlitePool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let ctx = Arc::new(ctx);
        let tasks = self.tasks.clone();
        let sirius = Sirius::new(self.client_id.clone(), self.client_secret.clone());

        tokio::spawn(async move {
            let mut tasks = tasks;
            let mut sirius = sirius;
            loop {
                check(Arc::clone(&ctx), &mut tasks, &mut sirius).await;
                tokio::time::sleep(Duration::from_secs(400)).await;
            }
        });
    }
}

async fn check(_ctx: Arc<Context>, tasks: &mut Vec<Task>, sirius: &mut Sirius) {
    println!("Tasks: {:?}", tasks);
    sirius.load_access_token().await.unwrap();
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD").expect("Expected Discord token in the environment");
    let client_id = env::var("CLIENT_ID").expect("Expected cilent id in the environment");
    let client_secret = env::var("CLIENT_SECRET").expect("Expected client secret in the environment");

    // TODO: Write default config if loading config fails

    // Write empty config
    // let conf: config::Config = config::Config::default();
    // write_config(&conf, Some(DEFAULT_PATH.into())).await;

    let path = DEFAULT_PATH;
    let mut config: Config = load_config(path).await;
    config.path = path.into();

    let db_options = SqliteConnectOptions::new()
        .filename(&config.db)
        .create_if_missing(true);

    let db_pool = SqlitePool::connect_with(db_options)
        .await
        .expect("Failed to connect to the SQLite database");

    // Convert config into tasks
    let tasks: Vec<Task> = config.into();

    let intents = GatewayIntents::GUILD_MESSAGES;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { client_id, client_secret, tasks , conn: db_pool })
        .await
        .expect("Error while creating the client!");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
