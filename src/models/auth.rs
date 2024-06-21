use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LoginPost {
    pub email: String,
    pub password: String,
}
