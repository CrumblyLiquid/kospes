use std::env;
use std::sync::Arc;
use std::time::Duration;

use dotenv::dotenv;

use serenity::all::GatewayIntents;
use serenity::prelude::*;
use serenity::{all::Ready, async_trait};

mod config;
use config::{load_config, write_config, Config};

const DEFAULT_PATH: &str = "./config.toml";

struct Handler {
    config: Config,
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
    println!("Objectives: {}", config.objectives.len());
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD").expect("Expected a token in the environment");

    // Write empty config
    // let conf: config::Config = config::Config::default();
    // write_config(DEFAULT_PATH, &conf).await;

    let config = load_config(DEFAULT_PATH).await;

    let intents = GatewayIntents::GUILD_MESSAGES;
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { config })
        .await
        .expect("Error while creating the client!");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
