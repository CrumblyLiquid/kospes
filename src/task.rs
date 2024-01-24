use serenity::model::id::{ChannelId, RoleId};
use crate::config::{Config, Metadata, DEFAULT_INTERVAL, DEFAULT_COOLDOWN};

#[derive(Debug, Clone)]
pub struct Task {
    pub name: String,
    pub event_type: String,
    pub interval: u32,
    pub cooldown: u32,
    pub channels: Vec<ChannelId>,
    pub pings: Vec<RoleId>,
}

impl Default for Task {
    fn default() -> Task {
        Task {
            name: String::default(),
            event_type: String::default(),
            interval: DEFAULT_INTERVAL,
            cooldown: DEFAULT_COOLDOWN,
            channels: Vec::default(),
            pings: Vec::default(),
        }
    }
}

fn apply_meta(task: &mut Task, meta: &Metadata) {
    if let Some(interval) = meta.interval {
        task.interval = interval;
    }
    if let Some(cooldown) = meta.cooldown {
        task.cooldown = cooldown;
    }
    if !meta.channels.is_empty() {
        task.channels = meta.channels.clone();
    }
    if !meta.pings.is_empty() {
        task.pings = meta.pings.clone();
    }
}

impl From<Config> for Vec<Task> {
    fn from(val: Config) -> Self {
        let mut tasks: Vec<Task> = Vec::new();
        for (name, subject) in val.subjects {
            for (event_type, event_meta) in subject.events {
                let mut task = Task {
                    name: name.clone(),
                    event_type,
                    ..Default::default()
                };

                // Try applying different levels of meta values
                // and the lowest one will stick
                apply_meta(&mut task, &val.meta);
                apply_meta(&mut task, &subject.meta);
                apply_meta(&mut task, &event_meta);

                tasks.push(task);
            }
        }

        tasks
    }
}
