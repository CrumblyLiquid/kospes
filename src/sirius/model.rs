use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

// This struct is used for deserializing a json response
// so we want to include all the variables even if we don't use them
#[allow(unused_variables)]
#[derive(Deserialize, Debug)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub scope: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventResult {
    pub meta: Meta,
    pub events: Vec<Event>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Meta {
    pub count: i32,
    pub offset: i32,
    pub limit: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub id: i32,
    pub name: Option<String>,
    pub sequence_number: i32,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub deleted: bool,
    pub capacity: i32,
    pub occupied: i32,
    /// Type of event
    /// Valid values:
    /// - assessment
    /// - course_event
    /// - exam
    /// - laboratory
    /// - lecture
    /// - tutorial
    pub event_type: String,
    pub parallel: String,
    pub original_data: OriginalData,
    pub links: Links,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OriginalData {
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub room_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Links {
    pub room: String,
    pub course: String,
    #[serde(default = "Vec::new")]
    pub teachers: Vec<String>,
    #[serde(default = "Vec::new")]
    pub students: Vec<String>,
    #[serde(default = "Vec::new")]
    pub applied_exceptions: Vec<i32>,
}
