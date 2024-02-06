use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use serenity::client::Context;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tokio::time::Duration;

use super::Database;
use crate::config::DEFAULT_INTERVAL;
use crate::{
    api::courses::{Courses, NewsOptions},
    config::Config,
};

pub async fn get_news_locks(
    ctx: Arc<Context>,
) -> Option<(
    Arc<RwLock<Config>>,
    Arc<RwLock<Courses>>,
    Arc<RwLock<SqlitePool>>,
)> {
    let data = ctx.data.read().await;

    let config_lock = data.get::<Config>()?.clone();
    let courses_lock = data.get::<Courses>()?.clone();
    let db_lock = data.get::<Database>()?.clone();

    Some((config_lock, courses_lock, db_lock))
}

pub async fn check_news(
    ctx: Arc<Context>,
    config_lock: Arc<RwLock<Config>>,
    courses_lock: Arc<RwLock<Courses>>,
    db_lock: Arc<RwLock<SqlitePool>>,
) -> Duration {
    println!("Checking news!");

    let mut courses_rc = courses_lock.write().await;
    let courses = courses_rc.deref_mut();
    match courses.news(NewsOptions::default()).await {
        Ok(news) => print!("News: {:#?}", news),
        Err(e) => println!("{:#?}", e),
    };

    // Determine the correct interval
    let config_rc = config_lock.read().await;
    let config = config_rc.deref();
    // Default interval
    let mut duration = DEFAULT_INTERVAL;
    // Global interval
    if let Some(interval) = config.meta.interval {
        duration = interval;
    }
    // News interval
    if let Some(news) = &config.news {
        if let Some(interval) = news.meta.interval {
            duration = interval;
        }
    }
    Duration::from_secs(duration.into())
}
