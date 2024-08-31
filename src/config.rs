use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::Result;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serenity::model::id::{ChannelId, RoleId};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use toml;

const DEFAULT_PATH: &str = "./config.toml";

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

    /// News settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub news: Option<News>,

    /// Map of subject names and their bodies
    #[serde(default)]
    #[serde(alias = "subject")]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub subjects: HashMap<String, Subject>,

    #[serde(default)]
    #[serde(alias = "calendar")]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub calendars: HashMap<String, Calendar>,

    // Fallback options in case more specific options are not present
    // We were flattening Option<Metadata> but it resulted
    // in Some(Metadata) with default (empty) fields even if it wasn't
    // present in the config
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Metadata::is_empty")]
    pub meta: Metadata,
}

/// Metadata to know how and when to post events
/// When optional values are not present, check for the values
/// of its parent and if that fails, use the default values
/// defined in config.rs
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    /// How often to check for updates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
    /// How long to wait for another check after update is detected
    #[serde(skip_serializing_if = "Option::is_none")]
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

impl Metadata {
    pub fn is_empty(&self) -> bool {
        self.interval.is_none()
            && self.cooldown.is_none()
            && self.channels.is_empty()
            && self.pings.is_empty()
    }

    pub fn apply(&mut self, meta: &Metadata) -> &mut Self {
        if self.interval.is_none() {
            self.interval = meta.interval;
        }

        if self.cooldown.is_none() {
            self.cooldown = meta.cooldown;
        }

        if self.channels.is_empty() {
            self.channels = meta.channels.clone();
        }

        if self.pings.is_empty() {
            self.pings = meta.pings.clone();
        }

        self
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            interval: Some(DEFAULT_INTERVAL),
            cooldown: Some(DEFAULT_COOLDOWN),
            channels: Vec::new(),
            pings: Vec::new()
        }
    }
}

/// Subject to watch via Sirius API
/// If an optional setting is not present, the global one is used
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Subject {
    /// Map of event types with their metadata
    #[serde(default)]
    #[serde(alias = "event")]
    pub events: HashMap<String, Metadata>,

    #[serde(flatten)]
    #[serde(skip_serializing_if = "Metadata::is_empty")]
    pub meta: Metadata,
}

/// Calendar in iCal format to pull special events from
/// If an optional setting is not present, the global one is used
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Calendar {
    /// Name of the calendar to use in messages
    // pub name: String,
    /// Path to the .ical file
    pub path: PathBuf,

    #[serde(flatten)]
    #[serde(skip_serializing_if = "Metadata::is_empty")]
    pub meta: Metadata,
}

/// News from Course pages
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct News {
    /// Which courses to watch
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub courses: HashMap<String, Metadata>,

    #[serde(flatten)]
    #[serde(skip_serializing_if = "Metadata::is_empty")]
    pub meta: Metadata,
}

pub async fn load_config(path: impl AsRef<Path>) -> Result<Config> {
    let mut file = fs::File::open(&path).await?;

    let mut config_str = String::new();
    file.read_to_string(&mut config_str).await?;

    let config = toml::from_str(&config_str)?;
    Ok(config)
}

// TODO: Maybe use toml_edit to preserve config formatting when saving
/// Write configuration to a file
/// Uses Config.path if path argument is not specified
/// Probably shouldn't be used since it doesn't keep any comments
/// or formatting that was there previously
pub async fn write_config(config: &Config, path: Option<PathBuf>) -> Result<PathBuf> {
    let cfg_path = match path {
        Some(ref path) => path,
        None => &config.path,
    };

    let config_str = toml::to_string(&config)?;
    let mut file = fs::File::create(cfg_path).await?;
    file.write(config_str.as_bytes()).await?;
    Ok(cfg_path.to_path_buf())
}

/// Tries to load config from the default path
/// If that fails, it constructs a Default config and
/// tries to write it on to the path
pub async fn get_config() -> Config {
    let path = DEFAULT_PATH;
    let config: Config = match fs::try_exists(path).await {
        Ok(true) => match load_config(path).await {
            Ok(mut conf) => {
                conf.path = path.into();
                conf
            }
            Err(e) => panic!("Failed to load config! Error: {}", e),
        },
        Ok(false) => {
            let conf: Config = Config::default();
            if let Err(e) = write_config(&conf, Some(path.into())).await {
                panic!("Failed to write default config! Error: {}", e)
            }
            conf
        }
        Err(e) => panic!("Failed to check config path! Error: {}", e),
    };

    config
}

pub fn get_env() -> (String, String, String) {
    // Environment variables
    // Maybe move them into config.toml?
    dotenv().ok();
    let token = env::var("DISCORD").expect("Expected Discord token in the environment");
    let client_id = env::var("CLIENT_ID").expect("Expected cilent id in the environment");
    let client_secret =
        env::var("CLIENT_SECRET").expect("Expected client secret in the environment");

    (token, client_id, client_secret)
}
