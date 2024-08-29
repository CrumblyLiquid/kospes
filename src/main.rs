mod api;
mod bot;
mod config;
mod task;
mod db;
mod string;

use bot::run;
use config::{get_config, get_env};

#[tokio::main]
async fn main() {
    // Load secrets from environment vars
    let (token, client_id, client_secret) = get_env();

    // Get config one way or another
    let config = get_config().await;

    // Setup and run the Discord bot
    run(config, client_id, client_secret, token).await;
}
