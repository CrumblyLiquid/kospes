use serenity::prelude::*;
use serenity::{all::Ready, async_trait};
use sqlx::sqlite::SqlitePool;
use std::sync::Arc;
use tokio::time::Duration;

use crate::api::{Api, Options};
use crate::task::Task;

#[allow(dead_code)]
pub struct Handler {
    client_id: String,
    client_secret: String,
    tasks: Vec<Task>,
    conn: SqlitePool,
}

impl Handler {
    pub fn new(
        client_id: String,
        client_secret: String,
        tasks: Vec<Task>,
        conn: SqlitePool,
    ) -> Self {
        Handler {
            client_id,
            client_secret,
            tasks,
            conn,
        }
    }
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
