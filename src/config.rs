use serde::{Deserialize, Serialize};
use serenity::model::id::{ChannelId, RoleId};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use toml;

// Database of previously seen events will be moved
// into its own sqlite database to keep config clean

/// Main configuration struct
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    // Where to save the config when we make changes
    // Not used, just future-proofing
    /// Location of this config (for saving)
    #[serde(skip)]
    pub path: PathBuf,

    // Probably should use the tokio-rusqlite crate for this
    /// Path to a sqlite database
    /// that is used for storing seen events
    pub db: PathBuf,

    pub subjects: Vec<Subject>,
    pub calendars: Vec<Calendar>,

    // Fallback options in case more specific options are not present
    /// How often to check for updates on subject
    pub interval: u32,
    /// Channel to post updates
    pub channel: Vec<ChannelId>,
    /// Role to ping when new updates are posted
    pub ping: Vec<RoleId>,
}

/// Metadata to know how and when to post events
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Metadata {
    /// How often to check for updates
    pub interval: Option<u32>,
    /// What channel to post events to
    pub channel: Vec<ChannelId>,
    /// What role to ping when new events are posted
    pub ping: Vec<RoleId>,
}

/// Subject to watch via Sirius API
/// If an optional setting is not present, the global one is used
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Subject {
    /// Name of the subject (as specified by Sirius API)
    pub name: String,
    /// What types of events should be watched
    pub events: Vec<Event>,
    pub meta: Metadata,
}

/// Specific event type
/// If an optional setting is not present, the global one is used
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Event {
    /// Name of the event type (as specified by Sirius API)
    pub event_type: String,
    pub meta: Metadata,
}

/// Calendar in iCal format to pull special events from
/// If an optional setting is not present, the global one is used
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Calendar {
    /// Name of the calendar to use in messages
    pub name: String,
    /// Path to the .ical file
    pub path: PathBuf,
    pub meta: Metadata,
}

// TODO: Return Result from load and write functions!

pub async fn load_config(path: impl AsRef<Path>) -> Config {
    let mut file = File::open(&path).await.expect("Failed to open config!");

    let mut config_str = String::new();
    file.read_to_string(&mut config_str)
        .await
        .expect("Failed to read config!");

    toml::from_str(&config_str).expect("Failed to parse config!")
}

/// Write configuration to a file
/// Uses Config.path if path argument is not specified
pub async fn write_config(config: &Config, path: Option<PathBuf>) {
    let cfg_path = match path {
        Some(ref path) => path,
        None => &config.path,
    };

    let config_str = toml::to_string(&config).expect("Faild to serialize config to string!");
    let mut file = File::open(cfg_path).await.expect("Failed to open config!");
    file.write(config_str.as_bytes())
        .await
        .expect("Failed to write config to file");
}
