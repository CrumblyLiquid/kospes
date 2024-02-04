use std::collections::HashMap;

mod auth;
pub mod sirius;
pub mod courses;

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
