use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

use super::auth::Auth;
use super::Options;

const COURSES_URL: &str = "https://courses.fit.cvut.cz/api/v1";

#[derive(Debug, Clone)]
pub struct Courses {
    auth: Auth,
}

impl Courses {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            auth: Auth::new(client_id, client_secret, "cvut:cpages:common:read".into()),
        }
    }

    // Doesn't work: Needs token with the correct scope -> Can't have generic Api :(
    pub async fn news(&mut self, options: NewsOptions) -> Result<Vec<News>> {
        let token = self.auth.get_token().await?;
        let map: HashMap<String, String> = options.with_token(&token);

        let url = format!("{}/cpages/news.json", COURSES_URL);

        let res = Client::new()
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .form(&map)
            .send()
            .await?;

        // TODO: Check for StatusCode::OK
        let text = res.text().await?;
        let content: Vec<News> = serde_json::from_str(&text)?;
        Ok(content)
    }
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
    pub uri: Option<String>,
}

#[derive(Debug, Default)]
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
