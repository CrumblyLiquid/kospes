use std::collections::HashMap;
use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};

mod model;
use model::EventResult;
mod auth;
use auth::Auth;

const URL: &str = "https://sirius.fit.cvut.cz/api/v1";

#[derive(Debug, Clone)]
pub struct Sirius {
    auth: Auth,
}

impl Sirius {
    pub fn new(client_id: String, client_secret: String) -> Sirius {
        Sirius {
            auth: Auth::new(client_id, client_secret),
        }
    }

    pub async fn course_events(
        &mut self,
        course_code: String,
        options: Options,
    ) -> Result<EventResult> {
        let token = self.auth.get_token().await?;

        let mut map: HashMap<String, String> = options.into();
        map.insert("access_token".into(), token);

        let url = format!("{}/courses/{}/events", URL, course_code);

        // We have to make request
        let res = Client::new().get(url).form(&map).send().await?;

        // TODO: Check for StatusCode::OK
        let text = res.text().await?;
        let content: EventResult = serde_json::from_str(&text)?;
        Ok(content)
    }
}

#[derive(Default, Debug)]
pub struct Options {
    /// The number of entries in collection to return
    /// Default: 10
    /// Maximum: 100
    pub limit: Option<u32>,
    /// Offset of the first entry in collection
    /// Default: 0
    pub offset: Option<u32>,
    /// A comma-separated list of the link names to include
    /// E.g.: `courses,teachers,schedule_exceptions`
    pub include: Option<String>,
    /// Filter by event's type
    pub event_type: Option<String>,
    /// Return even events that have been deleted
    /// Default: false
    pub deleted: Option<bool>,
    /// Return events from this date
    pub from: Option<DateTime<Utc>>,
    /// Return events up to this date
    pub to: Option<DateTime<Utc>>,
    /// When the date of event has been changed by a schedule exception,
    /// original date is not considered for date filtering (by from/to parameters).
    /// With this parameter Sirius will include events’ original date in a date filter.
    /// Default: false
    pub with_original_date: Option<bool>,
}

impl From<Options> for HashMap<String, String> {
    fn from(val: Options) -> Self {
        let mut map = HashMap::new();

        if let Some(limit) = val.limit {
            map.insert("limit".into(), limit.to_string());
        }

        if let Some(offset) = val.offset {
            map.insert("offset".into(), offset.to_string());
        }

        if let Some(include) = val.include {
            map.insert("include".into(), include);
        }

        if let Some(event_type) = val.event_type {
            map.insert("event_type".into(), event_type);
        }

        if let Some(deleted) = val.deleted {
            map.insert("deleted".into(), deleted.to_string());
        }

        if let Some(from) = val.from {
            map.insert("from".into(), from.to_string());
        }

        if let Some(to) = val.to {
            map.insert("to".into(), to.to_string());
        }

        if let Some(with_original_date) = val.with_original_date {
            map.insert("with_original_date".into(), with_original_date.to_string());
        }

        map
    }
}
