use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tokio::time::{Instant, Duration};

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

    // TODO: Better error checking (custom result type?)
    pub async fn load_access_token(&mut self) -> reqwest::Result<String> {
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
                let text = res.text().await.unwrap_or_else(|_| String::new());
                let content: AuthResponse =
                    serde_json::from_str(&text)
                    .expect("Failed to deserialize OAuth2 Response");
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
    ) -> Result<EventResult, ()> {
        let _token = self.load_access_token().await.expect("Failed to get token!");
        Err(())
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


pub struct Options {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub include: Option<String>,
    pub event_type: Option<String>,
    pub deleted: Option<bool>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub with_original_date: Option<bool>,
}

// TODO:
/*
impl Default for Options {

}
*/

pub struct EventResult {
    meta: Meta,
    events: Vec<Event>,
}
