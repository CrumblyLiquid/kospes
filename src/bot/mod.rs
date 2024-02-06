use std::sync::Arc;

use serenity::prelude::*;
use serenity::{all::Ready, async_trait};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use tokio::time::Duration;

use crate::api::courses::Courses;
use crate::api::sirius::{EventOptions, Sirius};
use crate::config::Config;
use crate::task::Task;

mod news;
mod events;

impl TypeMapKey for Config {
    type Value = Arc<RwLock<Config>>;
}

struct Tasks;
impl TypeMapKey for Tasks {
    type Value = Arc<RwLock<Vec<Task>>>;
}

struct Database;
impl TypeMapKey for Database {
    type Value = Arc<RwLock<SqlitePool>>;
}

impl TypeMapKey for Sirius {
    type Value = Arc<RwLock<Sirius>>;
}

impl TypeMapKey for Courses {
    type Value = Arc<RwLock<Courses>>;
}

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let ctx = Arc::new(ctx);
        /*
        tokio::spawn(async move {
            loop {
                let (sirius_lock, courses_lock, task_lock, db_lock) = get_locks(Arc::clone(&ctx))
                    .await
                    .expect("Faild to obtain all locks from TypeMap");

                let duration = events::check_events(
                    Arc::clone(&ctx),
                    sirius_lock,
                    task_lock,
                    db_lock,
                )
                .await;

                tokio::time::sleep(duration).await;
            }
        });
        */

        let ctx = Arc::clone(&ctx);
        tokio::spawn(async move {
            loop {
                println!("News loop started!");

                let (config_lock, courses_lock, db_lock) = news::get_news_locks(Arc::clone(&ctx))
                    .await
                    .expect("Faild to obtain all locks from TypeMap");

                let duration = news::check_news(
                    Arc::clone(&ctx),
                    config_lock,
                    courses_lock,
                    db_lock,
                )
                .await;

                tokio::time::sleep(duration).await;
            }
        });
    }
}


pub async fn run(config: Config, client_id: String, client_secret: String, token: String) {
    // Create SQLite database connection
    // Used for storing seen events, etc.
    let db_options = SqliteConnectOptions::new()
        .filename(&config.db)
        .create_if_missing(true);

    let db_pool = SqlitePool::connect_with(db_options)
        .await
        .expect("Failed to connect to the SQLite database");
    let tasks: Vec<Task> = config.clone().into();
    let sirius: Sirius = Sirius::new(client_id.clone(), client_secret.clone());
    let courses: Courses = Courses::new(client_id, client_secret);

    let intents = GatewayIntents::GUILD_MESSAGES;
    let mut client = Client::builder(&token, intents)
        .type_map_insert::<Config>(Arc::from(RwLock::from(config)))
        .type_map_insert::<Sirius>(Arc::from(RwLock::from(sirius)))
        .type_map_insert::<Courses>(Arc::from(RwLock::from(courses)))
        .type_map_insert::<Tasks>(Arc::from(RwLock::from(tasks)))
        .type_map_insert::<Database>(Arc::from(RwLock::from(db_pool)))
        .event_handler(Bot)
        .await
        .expect("Error while creating the client!");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
