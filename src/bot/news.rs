use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use anyhow::Result;
use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage};
use serenity::client::Context;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tokio::time::Duration;

use super::Database;
use crate::api::courses::News;
use crate::config::DEFAULT_INTERVAL;
use crate::{
    api::courses::{Courses, NewsOptions},
    config::Config,
};

async fn get_news_locks(
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

async fn get_duration(config_lock: Arc<RwLock<Config>>) -> Duration {
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

/// Get list of courses we want to recieve news about
async fn get_subjects(config_lock: Arc<RwLock<Config>>) -> Option<Vec<String>> {
    let config_rc = config_lock.read().await;
    let config = config_rc.deref();

    if let Some(news) = &config.news {
        Some(news.courses.clone())
    } else {
        None
    }
}

async fn get_news(
    courses_lock: Arc<RwLock<Courses>>,
    subjects: Option<Vec<String>>
) -> Result<HashMap<String, Vec<News>>> {
    let mut courses_rc = courses_lock.write().await;
    let courses = courses_rc.deref_mut();

    let options = NewsOptions {
        representation: Some("grouped".into()),
        courses: subjects,
        ..Default::default()
    };

    courses.news(options).await
}

async fn get_new_news(
    db_lock: Arc<RwLock<SqlitePool>>,
    news: HashMap<String, Vec<News>>,
) -> Vec<News> {
    // println!("News: {:#?}", news);
    let mut new_news: Vec<News> = Vec::new();

    {
        let db_rc = db_lock.read().await;
        let db = db_rc.deref();

        for (_subject, news) in news {
            for new in news {
                if new.is_new(db).await {
                    println!("New: {:#?}", new);
                    new_news.push(new);
                }
            }
        }
    }

    if new_news.is_empty() {
        println!("No new news!");
    }

    new_news
}

async fn post_news(new_news: Vec<News>, ctx: Arc<Context>) {
    let mut messages: Vec<CreateMessage> = Vec::new();
    let mut last_vec: Vec<News> = Vec::new();

    for new in new_news {
        println!("Posting: {:#?}", new);
    }
}

pub async fn check_news(ctx: Arc<Context>) -> Duration {
    println!("Checking news!");

    let (config_lock, courses_lock, db_lock) = get_news_locks(Arc::clone(&ctx))
        .await
        .expect("Failed to obtain all locks from Bot's TypeMap");

    let subjects = get_subjects(Arc::clone(&config_lock)).await;

    match get_news(courses_lock, subjects).await {
        Ok(news) => {
            println!("Gotten news");
            for (subject, news_vec) in &news {
                println!("{}: {}", subject, news_vec.len());
            }
            let new_news = get_new_news(db_lock, news).await;
            post_news(new_news, Arc::clone(&ctx)).await;
        }
        Err(e) => println!("Failed to obtain News: {:#?}", e),
    }

    get_duration(config_lock).await
}

impl News {
    pub async fn is_new(&self, pool: &SqlitePool) -> bool {
        let res = sqlx::query_as::<_, (u32,)>("SELECT COUNT(*) FROM seen_news WHERE news_id = $1")
            .bind(&self.id)
            .fetch_one(pool)
            .await;

        match res {
            Ok(count) => {
                if let Err(e) = sqlx::query("INSERT INTO seen_news (news_id) VALUES ($1);")
                    .bind(&self.id.clone())
                    .execute(pool)
                    .await
                {
                    println!(
                        "Failed to add News (id: {}) to seen_news table: {e}",
                        self.id
                    )
                };

                if count.0 > 0 {
                    false
                } else {
                    true
                }
            }
            Err(e) => {
                println!("Failed to run News::is_new SQL query: {e}");
                true
            }
        }
    }
}
