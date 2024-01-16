use chrono::DateTime;

pub enum EventType {
    Assesment,
    CourseEvent,
    Exam,
    Laboratory,
    Lecture,
    Tutorial,
}

impl EventType {
    fn as_str(&self) -> &'static str {
        match self {
            EventType::Assesment => "assessment",
            EventType::CourseEvent => "course_event",
            EventType::Exam => "exam",
            EventType::Laboratory => "laboratory",
            EventType::Lecture => "lecture",
            EventType::Tutorial => "tutorial",
        }
    }
}

pub struct Meta {
    count: i32,
    offset: i32,
    limit: i32,
}

pub struct Event {
    pub id: i32,
    pub name: Option<String>,
    pub sequence_number: i32,
    pub starts_at: DateTime,
    pub ends_at: DateTime,
    pub deleted: bool,
    pub capacity: i32,
    pub occupied: i32,
    pub event_type: EventType,
    pub parallel: String,
    pub original_data: OriginalData,
    pub links: Links,
}

pub struct OriginalData {
    pub starts_at: Option<DateTime>,
    pub ends_at: Option<DateTime>,
    pub room_id: Option<String>,
}

pub struct Links {
    pub room: String,
    pub course: String,
    pub teachers: Vec<String>,
    pub students: Vec<String>,
    pub applied_exceptions: Vec<i32>,
}
