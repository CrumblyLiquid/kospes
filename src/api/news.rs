use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;

use super::Options;

#[derive(Deserialize, Debug)]
pub struct NewsResponse {
    #[serde(flatten)]
    pub items: Vec<News>,
}

#[derive(Deserialize, Debug)]
pub struct NewsGroupedResponse {
    #[serde(flatten)]
    pub items: HashMap<String, Vec<News>>,
}

#[derive(Deserialize, Debug)]
pub struct News {
    pub id: String,
    pub title: String,
    pub content: String,

    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "createdBy")]
    pub created_by: NewsAuthor,

    #[serde(rename = "modifiedAt")]
    pub modified_at: Option<DateTime<Utc>>,
    #[serde(rename = "modifiedBy")]
    pub modified_by: Option<NewsAuthor>,

    #[serde(rename = "publishedAt")]
    pub published_at: DateTime<Utc>,

    #[serde(rename = "ref")]
    pub git_ref: String,
    pub deleted: bool,
    pub audience: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct NewsAuthor {
    pub name: String,
    pub uri: String,
}

#[derive(Debug)]
pub struct NewsOptions {
    /// Specify type of the JSON representation:
    /// Default: `default`
    /// Enum:
    ///     `default` - an array of messages
    ///     `grouped` - messages grouped by audience
    ///     `jsonfeed` - messages in the JSON Feed format
    pub representation: Option<String>,
    /// Return messages only from the specified course pages
    /// Example: `courses=BI-LIN,MI-RUB`
    pub courses: Option<Vec<String>>,
    /// Whether to return even deleted messages
    /// Default: false
    pub deleted: Option<bool>,
    /// Maximum number of messages to return.
    /// If `type=grouped`, this parameter limits number of the messages per course
    pub limit: Option<u32>,
    /// Offset of the first message to return.
    /// This parameter is ignored when `type=grouped`
    /// Default: 0
    pub offset: Option<u32>,
    /// Return messages published since this date
    /// Example: `since=2019-06-01`
    pub since: Option<DateTime<Utc>>,
    /// Return messages published until this date
    /// Example: `until=2019-06-01`
    pub until: Option<DateTime<Utc>>,
}

impl From<NewsOptions> for HashMap<String, String> {
    fn from(val: NewsOptions) -> Self {
        let mut map = HashMap::new();

        if let Some(representation) = val.representation {
            map.insert("type".into(), representation);
        }

        if let Some(courses) = val.courses {
            map.insert("courses".into(), courses.join(","));
        }

        if let Some(deleted) = val.deleted {
            map.insert("deleted".into(), deleted.to_string());
        }

        if let Some(limit) = val.limit {
            map.insert("limit".into(), limit.to_string());
        }

        if let Some(offset) = val.offset {
            map.insert("offset".into(), offset.to_string());
        }

        if let Some(since) = val.since {
            map.insert("since".into(), since.to_string());
        }

        if let Some(until) = val.until {
            map.insert("until".into(), until.to_string());
        }

        map
    }
}

impl Options for NewsOptions {}
