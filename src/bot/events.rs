use std::sync::Arc;

use serenity::client::Context;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use tokio::time::Duration;

use super::{Database, Tasks};
use crate::{api::sirius::Sirius, config::Config, task::Task};

async fn get_sirius_locks(
    ctx: Arc<Context>,
) -> Option<(
    Arc<RwLock<Config>>,
    Arc<RwLock<Sirius>>,
    Arc<RwLock<SqlitePool>>,
)> {
    let data = ctx.data.read().await;

    let config_lock = data.get::<Config>()?.clone();
    let sirius_lock = data.get::<Sirius>()?.clone();
    let db_lock = data.get::<Database>()?.clone();

    Some((config_lock, sirius_lock, db_lock))
}
