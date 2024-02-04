use dotenv::dotenv;
use std::env;
use tokio::fs;

mod api;
mod bot;
mod config;
mod task;

use bot::run;
use config::{load_config, write_config, Config};

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
    run(config, client_id, client_secret, token).await;
}
