use std::env;
use std::sync::Arc;
use tokio::time::Duration;
use tokio::fs;
use dotenv::dotenv;
use serenity::all::GatewayIntents;
use serenity::prelude::*;
use serenity::{all::Ready, async_trait};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};

mod config;
use config::{load_config, write_config, Config};
mod task;
use task::Task;
mod api;
use api::{Api, Options};

const DEFAULT_PATH: &str = "./config.toml";

struct Handler {
    client_id: String,
    client_secret: String,
    tasks: Vec<Task>,
    _conn: SqlitePool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let ctx = Arc::new(ctx);
        let tasks = self.tasks.clone();
        let api = Api::new(self.client_id.clone(), self.client_secret.clone());

        tokio::spawn(async move {
            let mut tasks = tasks;
            let mut api = api;
            loop {
                check(Arc::clone(&ctx), &mut tasks, &mut api).await;
                tokio::time::sleep(Duration::from_secs(400)).await;
            }
        });
    }
}

async fn check(_ctx: Arc<Context>, tasks: &mut Vec<Task>, sirius: &mut Api) {
    println!("Tasks: {:#?}", tasks);
    let opts = Options {
        ..Options::default()
    };
    let res = sirius.course_events("BI-LA1.21".into(), opts).await;
    println!("Response: {:#?}", res);
}

#[tokio::main]
async fn main() {
    // Environment variables
    // Maybe move them into config.toml?
    dotenv().ok();
    let token =
        env::var("DISCORD").expect("Expected Discord token in the environment");
    let client_id =
        env::var("CLIENT_ID").expect("Expected cilent id in the environment");
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

    // Convert config into tasks
    let tasks: Vec<Task> = config.into();

    let intents = GatewayIntents::GUILD_MESSAGES;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            client_id,
            client_secret,
            tasks,
            _conn: db_pool,
        })
        .await
        .expect("Error while creating the client!");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
