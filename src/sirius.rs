const URL: &str = "https://sirius.fit.cvut.cz/api/v1";

#[derive(Debug, Default, Clone)]
struct Sirius {
    client_id: String,
    client_secret: String,
    access_token: Option<String>,
    expires_int: i32
}

impl Sirius {
    async fn get_access_token(&self) {

    }
}
