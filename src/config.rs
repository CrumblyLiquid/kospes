use anyhow::Result;
use serde::{Deserialize, Serialize};
use serenity::model::id::{ChannelId, RoleId};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use toml;

pub const DEFAULT_INTERVAL: u32 = 2 * 60 * 60; // 2 hours
pub const DEFAULT_COOLDOWN: u32 = 24 * 60 * 60; // 1 day

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

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub subjects: Vec<Subject>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub calendars: Vec<Calendar>,

    // Fallback options in case more specific options are not present
    #[serde(flatten)]
    pub meta: Option<Metadata>,
}

/// Metadata to know how and when to post events
/// When optional values are not present, check for the values
/// of its parent and if that fails, use the default values
/// defined in config.rs
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Metadata {
    /// How often to check for updates
    pub interval: Option<u32>,
    /// How long to wait for another check after update is detected
    pub cooldown: Option<u32>,
    /// What channel to post events to
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub channels: Vec<ChannelId>,
    /// What role to ping when new events are posted
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub pings: Vec<RoleId>,
}

/// Subject to watch via Sirius API
/// If an optional setting is not present, the global one is used
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Subject {
    /// Name of the subject (as specified by Sirius API)
    pub name: String,
    /// What types of events should be watched
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(flatten)]
    pub meta: Option<Metadata>,
}

/// Specific event type
/// If an optional setting is not present, the global one is used
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Event {
    /// Name of the event type (as specified by Sirius API)
    pub event_type: String,
    #[serde(flatten)]
    pub meta: Option<Metadata>,
}

/// Calendar in iCal format to pull special events from
/// If an optional setting is not present, the global one is used
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Calendar {
    /// Name of the calendar to use in messages
    pub name: String,
    /// Path to the .ical file
    pub path: PathBuf,
    #[serde(flatten)]
    pub meta: Option<Metadata>,
}

pub async fn load_config(path: impl AsRef<Path>) -> Result<Config> {
    let mut file = File::open(&path).await?;

    let mut config_str = String::new();
    file.read_to_string(&mut config_str).await?;

    let config = toml::from_str(&config_str)?;
    Ok(config)
}

/// Write configuration to a file
/// Uses Config.path if path argument is not specified
pub async fn write_config(config: &Config, path: Option<PathBuf>) -> Result<PathBuf> {
    let cfg_path = match path {
        Some(ref path) => path,
        None => &config.path,
    };

    let config_str = toml::to_string(&config)?;
    let mut file = File::create(cfg_path).await?;
    file.write(config_str.as_bytes()).await?;
    Ok(cfg_path.to_path_buf())
}
