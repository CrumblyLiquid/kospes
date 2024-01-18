use anyhow::Result;
use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};

mod model;
use model::{Event, Meta};

const URL: &str = "https://sirius.fit.cvut.cz/api/v1";
const OAUTH_URL: &str = "https://auth.fit.cvut.cz/oauth/oauth/token";

#[derive(Debug, Clone)]
pub struct Sirius {
    client_id: String,
    client_secret: String,
    access_token: Option<String>,
    expires_in: Instant,
}

impl Sirius {
    pub fn new(client_id: String, client_secret: String) -> Sirius {
        Sirius {
            client_id,
            client_secret,
            access_token: None,
            expires_in: Instant::now(),
        }
    }

    pub async fn load_access_token(&mut self) -> Result<String> {
        let now = Instant::now();
        if self.access_token.is_none() || self.expires_in <= now {
            // We have to make request
            let res = Client::new()
                .post(OAUTH_URL)
                .form(&[
                    ("grant_type", "client_credentials"),
                    ("client_id", &self.client_id),
                    ("client_secret", &self.client_secret),
                    ("scope", "cvut:sirius:personal:read"),
                ])
                .send()
                .await?;

            if res.status() == StatusCode::OK {
                let text = res.text().await?;
                let content: AuthResponse = serde_json::from_str(&text)?;
                self.access_token = Some(content.access_token.clone());
                self.expires_in = Instant::now() + Duration::from_secs(content.expires_in);
            }
        }
        Ok(self.access_token.clone().unwrap())
    }

    pub async fn course_events(
        &mut self,
        _course_code: String,
        _options: Options,
    // TODO: Call API and return EventResult
    ) -> Result<String> {
        self.load_access_token().await
    }
}

pub enum SiriusError {}

// This struct is used for deserializing a json response
// so we want to include all the variables even if we don't use them
#[allow(unused_variables)]
#[derive(Deserialize, Debug)]
struct AuthResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    scope: String,
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

#[derive(Serialize, Deserialize, Debug)]
pub struct EventResult {
    #[serde(flatten)]
    meta: Meta,
    events: Vec<Event>,
}
