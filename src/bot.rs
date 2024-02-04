use serenity::prelude::*;
use serenity::{all::Ready, async_trait};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool};
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::time::Duration;

use crate::api::{event::EventOptions, Api};
use crate::config::Config;
use crate::task::Task;

struct Tasks;
impl TypeMapKey for Tasks {
    type Value = Arc<RwLock<Vec<Task>>>;
}

struct Database;
impl TypeMapKey for Database {
    type Value = Arc<RwLock<SqlitePool>>;
}

impl TypeMapKey for Api {
    type Value = Arc<RwLock<Api>>;
}

struct Bot;

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let ctx = Arc::new(ctx);
        tokio::spawn(async move {
            loop {
                let (api_lock, task_lock, db_lock) = get_locks(Arc::clone(&ctx))
                    .await
                    .expect("Faild to obtain all locks from TypeMap");

                let duration = check(Arc::clone(&ctx), api_lock, task_lock, db_lock).await;

                tokio::time::sleep(duration).await;
            }
        });
    }
}

async fn get_locks(
    ctx: Arc<Context>,
) -> Option<(
    Arc<RwLock<Api>>,
    Arc<RwLock<Vec<Task>>>,
    Arc<RwLock<SqlitePool>>,
)> {
    let data = ctx.data.read().await;

    let api_lock = data.get::<Api>()?.clone();
    let tasks_lock = data.get::<Tasks>()?.clone();
    let db_lock = data.get::<Database>()?.clone();

    Some((api_lock, tasks_lock, db_lock))
}

async fn check(
    ctx: Arc<Context>,
    api_lock: Arc<RwLock<Api>>,
    task_lock: Arc<RwLock<Vec<Task>>>,
    db_lock: Arc<RwLock<SqlitePool>>,
) -> Duration {
    let mut api_rc = api_lock.write().await;
    let api = api_rc.deref_mut();
    if let Ok(events) = api
        .course_events("BI-LA1.21".into(), EventOptions::default())
        .await
    {
        print!("{:#?}", events);
    }

    Duration::from_secs(1)
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

    let tasks: Vec<Task> = config.into();

    let intents = GatewayIntents::GUILD_MESSAGES;
    let mut client = Client::builder(&token, intents)
        .type_map_insert::<Api>(Arc::from(RwLock::from(Api::new(client_id, client_secret))))
        .type_map_insert::<Tasks>(Arc::from(RwLock::from(tasks)))
        .type_map_insert::<Database>(Arc::from(RwLock::from(db_pool)))
        .event_handler(Bot)
        .await
        .expect("Error while creating the client!");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
