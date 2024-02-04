use anyhow::Result;
use reqwest::{Client, StatusCode};
use std::collections::HashMap;

mod auth;
use auth::Auth;
pub mod event;
use event::{EventOptions, EventsResponse};
pub mod news;
use news::{NewsOptions, NewsResponse};

const SIRIUS_URL: &str = "https://sirius.fit.cvut.cz/api/v1";
const COURSES_URL: &str = "https://courses.fit.cvut.cz/api/v1";

#[derive(Debug, Clone)]
pub struct Api {
    auth: Auth,
}

impl Api {
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            auth: Auth::new(client_id, client_secret),
        }
    }

    pub async fn course_events(
        &mut self,
        course_code: String,
        options: EventOptions,
    ) -> Result<EventsResponse> {
        let token = self.auth.get_token().await?;
        let map: HashMap<String, String> = options.with_token(token);

        let url = format!("{}/courses/{}/events", SIRIUS_URL, course_code);

        // We have to make request
        let res = Client::new().get(url).form(&map).send().await?;

        // TODO: Check for StatusCode::OK
        let text = res.text().await?;
        let content: EventsResponse = serde_json::from_str(&text)?;
        Ok(content)
    }

    // Doesn't work: Needs token with the correct scope -> Can't have generic Api :(
    pub async fn cpages_news(&mut self, options: NewsOptions) -> Result<NewsResponse> {
        let token = self.auth.get_token().await?;
        let map: HashMap<String, String> = options.with_token(token);

        let url = format!("{}/cpages/news.json", COURSES_URL);

        // We have to make request
        let res = Client::new().get(url).form(&map).send().await?;

        // TODO: Check for StatusCode::OK
        let text = res.text().await?;
        let content: NewsResponse = serde_json::from_str(&text)?;
        Ok(content)
    }
}

pub trait Options {
    fn with_token(self: Self, access_token: String) -> HashMap<String, String>
    where
        Self: Sized,
        Self: Into<HashMap<String, String>>,
    {
        let mut map: HashMap<String, String> = self.into();
        map.insert("access_token".into(), access_token);
        map
    }
}
