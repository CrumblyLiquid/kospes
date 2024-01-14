use serenity::model::id::{RoleId, ChannelId};
use serde::{Serialize,Deserialize};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use std::path::Path;
use toml;

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub objectives: Vec<Objective>,
    // Seen events will be moved into a sqlite database as they shouldn't
    // really appear in configuration
    // seen_events: Vec<>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Objective {
    pub name: String,
    pub event_type: String,
    pub channel: ChannelId,
    pub ping: RoleId
}

pub async fn load_config(path: impl AsRef<Path>) -> Config {
    let mut file = File::open(&path).await.expect("Failed to open config!");

    let mut config_str = String::new();
    file.read_to_string(&mut config_str).await.expect("Failed to read config!");

    toml::from_str(&config_str).expect("Failed to parse config!")
}

pub async fn write_config(path: impl AsRef<Path>, config: &Config) {
    let config_str = toml::to_string(&config).expect("Faild to serialize config to string!");
    let mut file = File::open(&path).await.expect("Failed to open config!");
    file.write(config_str.as_bytes()).await.expect("Failed to write config to file");
}
