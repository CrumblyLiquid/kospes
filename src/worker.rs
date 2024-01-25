use std::sync::Arc;

use serenity::client::Context;
use serenity::prelude::TypeMapKey;
use tokio::sync::RwLock;
use tokio::time::Duration;

use crate::api::Api;
use crate::task::Task;

// TODO: sqlite database 
pub struct Worker {
    api: Api,
    tasks: Vec<Task>,
}

impl<'a> Worker {
    pub fn new(client_id: String, client_secret: String, tasks: Vec<Task>) -> Self {
        Self {
            api: Api::new(client_id, client_secret),
            tasks,
        }
    }

    /// Runs the worker loop until there's no Tasks to consume
    /// anymore and returns the amount to sleep for
    pub async fn run(self, ctx: Arc<Context>) -> Duration {
        // For every task that should've been executed
        // Fetch and post new events
        Duration::from_secs(1)
    }

    fn get(self) -> Option<&'a Task> {
        None
    }

    fn sort_queue(self) {}
}

impl TypeMapKey for Worker {
    type Value = Arc<RwLock<Worker>>;
}
