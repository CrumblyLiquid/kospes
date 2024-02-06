use anyhow::Result;
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tokio::time::{Duration, Instant};

const OAUTH_URL: &str = "https://auth.fit.cvut.cz/oauth/oauth/token";

#[derive(Debug, Clone)]
pub struct Auth {
    client_id: String,
    client_secret: String,
    scope: String,
    pub access_token: Option<String>,
    pub expires_in: Instant,
}

impl Auth {
    pub fn new(client_id: String, client_secret: String, scope: String) -> Auth {
        Auth {
            client_id,
            client_secret,
            scope,
            access_token: None,
            expires_in: Instant::now(),
        }
    }

    pub async fn get_token(&mut self) -> Result<String> {
        let now = Instant::now();
        if self.access_token.is_none() || self.expires_in <= now {
            // We have to make request
            let res = Client::new()
                .post(OAUTH_URL)
                .form(&[
                    ("grant_type", "client_credentials"),
                    ("client_id", &self.client_id),
                    ("client_secret", &self.client_secret),
                    ("scope", &self.scope),
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
        Ok(self.access_token.clone().expect("Access token was None even after fetching it"))
    }
}

// This struct is used for deserializing a json response
// so we want to include all the variables even if we don't use them
#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct AuthResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    scope: String,
}
