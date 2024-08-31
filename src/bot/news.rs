use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use anyhow::Result;
use serenity::all::{Colour, Mentionable, Timestamp};
use serenity::builder::{CreateEmbed, CreateEmbedAuthor, CreateMessage};
use serenity::client::Context;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tokio::time::Duration;

use super::Database;
use crate::api::courses::News;
use crate::config::{get_default_meta, Metadata};
use crate::{
    api::courses::{Courses, NewsOptions},
    config::Config,
    string::ToMarkdown,
};

/// Obtian needed locks from the Bot's data storage
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

/// Get the duration until the next check (of new News)
async fn get_duration(config_lock: Arc<RwLock<Config>>, new_news: bool) -> Duration {
    // Determine the correct interval
    let config_rc = config_lock.read().await;
    let config = config_rc.deref();

    // Default interval
    let mut meta = Metadata::default();
    meta.apply(&config.meta);

    // News interval
    if let Some(news) = &config.news {
        meta.apply(&news.meta);
    }

    let duration = meta.interval.expect("No default interval found!");
    if new_news {
        duration.saturating_add(meta.cooldown.expect("No default cooldown found!"));
    }

    Duration::from_secs(duration.into())
}

/// Get list of courses we want to recieve news about
async fn get_subjects(config_lock: Arc<RwLock<Config>>) -> HashMap<String, Metadata> {
    let config_rc = config_lock.read().await;
    let config = config_rc.deref();

    let default_meta = &config.meta;

    let subjects = if let Some(news) = &config.news {
        let mut news_meta = news.meta.clone();
        news_meta.apply(default_meta);

        let mut subjects = news.courses.clone();
        for (_subject, meta) in &mut subjects {
            meta.apply(&news_meta);
        }

        subjects.insert("default".to_owned(), news_meta);
        subjects
    } else {
        HashMap::from_iter([("default".to_owned(), default_meta.clone())])
    };

    subjects
}

async fn get_news(
    courses_lock: Arc<RwLock<Courses>>,
    subjects: &HashMap<String, Metadata>,
) -> Result<HashMap<String, Vec<News>>> {
    let news: HashMap<String, Vec<News>> = {
        let mut courses_rc = courses_lock.write().await;
        let courses = courses_rc.deref_mut();

        let options = NewsOptions {
            representation: Some("grouped".into()),
            courses: Some(subjects.keys().cloned().collect()),
            // courses: None,
            ..Default::default()
        };

        courses.news(options).await?
    };

    // Filter by selected subject as the request for e.g. BI-MA1.21
    // will also return BIK-MA1.21 for some reason
    Ok(news
        .into_iter()
        .filter(|(subject, _news)| subjects.contains_key(subject))
        .collect())
}

/// Filters News to only contain the ones we haven't seen before
async fn get_new_news(
    db_lock: Arc<RwLock<SqlitePool>>,
    news: HashMap<String, Vec<News>>,
) -> HashMap<String, Vec<News>> {
    // println!("News: {:#?}", news);

    // News sorted by Subject that we haven't seen before
    // and therefore should send a message announcing them
    let mut new_news: HashMap<String, Vec<News>> = HashMap::new();

    // Limit how long we're keeping the db lock
    {
        let db_rc = db_lock.read().await;
        let db = db_rc.deref();

        for (subject, news) in news {
            println!("Checking: {}", &subject);

            // New news only for this subject
            let mut new_subject_news: Vec<News> = Vec::new();

            // Check what news are new
            // (if they are new, they're automatically
            // added to the database)
            for new in news {
                if new.is_new(db).await.unwrap_or(false) {
                    println!("New: {}", &new.id);
                    new_subject_news.push(new);
                }
            }

            // Only add subject if there are any new News
            if !new_subject_news.is_empty() {
                println!("Inserted {}", &subject);
                new_news.insert(subject, new_subject_news);
            }
        }
    }

    if new_news.is_empty() {
        println!("No new news!");
    }

    new_news
}

async fn post_news(
    ctx: Arc<Context>,
    new_news: HashMap<String, Vec<News>>,
    subjects: &HashMap<String, Metadata>,
) {
    let default_meta = subjects
        .get("default")
        .expect("No default subject meta present in Subjects HashMap");

    for (subject, news) in &new_news {
        println!("Subject: {} ({})", subject, news.len());
        // If no Metadata is present
        let meta = subjects.get(subject).unwrap_or(default_meta);

        // Create embeds
        let mut embeds: Vec<CreateEmbed> = Vec::new();
        for new in news {
            // println!("Posting: {}", new.id);
            let embed: CreateEmbed = new.embed(subject);
            embeds.push(embed);
        }

        // Construct text message which includes the correct pings
        let mut content = String::from("Novinky z ");
        content.push_str(subject);
        for role in &meta.pings {
            let mention = format!(" {}", role.mention());
            content.push_str(&mention);
        }

        // Create messages
        let mut messages: Vec<CreateMessage> = Vec::new();
        // A Discord message can contain at most 10 embeds
        for chunk in embeds.chunks(10) {
            let message: CreateMessage = CreateMessage::new()
                // TODO: Pings
                .content(content.clone())
                .add_embeds(chunk.to_vec());
            messages.push(message);
        }

        // Post messages
        for channel in &meta.channels {
            for message in &messages {
                match channel.send_message(&ctx.http, message.clone()).await {
                    Ok(_msg) => println!("Sent News from {} to {:?}", subject, channel),
                    Err(e) => println!(
                        "Failed to send News from {} to {:?}: {:#?}",
                        subject, channel, e
                    ),
                }
            }
        }
    }
}

/// Check Courses for any new News
/// If there are some it sends a message about the News
pub async fn check_news(ctx: Arc<Context>) -> Duration {
    println!("Checking news!");

    let (config_lock, courses_lock, db_lock) = get_news_locks(Arc::clone(&ctx))
        .await
        .expect("Failed to obtain all locks from Bot's TypeMap");

    let subjects = get_subjects(Arc::clone(&config_lock)).await;

    let are_new_news: bool = if !subjects.is_empty() {
        match get_news(courses_lock, &subjects).await {
            Ok(news) => {
                println!("Gotten news");
                for (subject, news_vec) in &news {
                    println!("{}: {}", subject, news_vec.len());
                }
                let new_news = get_new_news(db_lock, news).await;
                // Only post news if there are any to post
                if !new_news.is_empty() {
                    post_news(Arc::clone(&ctx), new_news, &subjects).await;
                }
                true
            }
            Err(e) => {
                println!("Failed to obtain News: {:#?}", e);
                false
            }
        }
    } else {
        false
    };

    get_duration(config_lock, are_new_news).await
}

impl News {
    /// Checks the DB if the News are new (if we haven't seen them yet)
    /// If we haven't seen them, it adds them to the DB
    pub async fn is_new(&self, pool: &SqlitePool) -> Option<bool> {
        let res =
            sqlx::query_as::<_, (u32,)>("SELECT EXISTS (SELECT * FROM seen_news WHERE id = $1)")
                .bind(&self.id)
                .fetch_one(pool)
                .await;

        match res {
            Ok(count) => {
                // News with that ID in the database doesn't exists
                if count.0 == 0 {
                    // So we should add it into the database
                    if let Err(e) = sqlx::query("INSERT INTO seen_news (id) VALUES ($1);")
                        .bind(&self.id.clone())
                        .execute(pool)
                        .await
                    {
                        println!(
                            "Failed to add News (id: {}) to seen_news table: {e}",
                            self.id
                        );

                        return None;
                    };

                    Some(true)
                // There is a row with that ID so this News isn't new
                } else {
                    Some(false)
                }
            }
            Err(e) => {
                println!("Failed to run News::is_new SQL query: {e}");
                None
            }
        }
    }

    /// Returns a Serenity Embed that represents the News
    /// that can be directly sent to any Channel
    pub fn embed(&self, subject: &str) -> CreateEmbed {
        CreateEmbed::new()
            .title(format!("{}: {}", subject, &self.title))
            .description(self.content.to_md(subject))
            .colour(Colour::from_rgb(0, 112, 186))
            .author(CreateEmbedAuthor::new(self.created_by.name.clone()))
            .timestamp(Timestamp::from(self.created_at))
    }
}
