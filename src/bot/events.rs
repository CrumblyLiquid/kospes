use std::sync::Arc;

use serenity::client::Context;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tokio::time::Duration;

use super::{Database, Tasks};
use crate::{api::sirius::Sirius, task::Task};

async fn get_sirius_locks(
    ctx: Arc<Context>,
) -> Option<(
    Arc<RwLock<Sirius>>,
    Arc<RwLock<Vec<Task>>>,
    Arc<RwLock<SqlitePool>>,
)> {
    let data = ctx.data.read().await;

    let sirius_lock = data.get::<Sirius>()?.clone();
    let tasks_lock = data.get::<Tasks>()?.clone();
    let db_lock = data.get::<Database>()?.clone();

    Some((sirius_lock, tasks_lock, db_lock))
}
