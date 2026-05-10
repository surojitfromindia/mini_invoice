use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database_url: String,
    pub login_secret_pepper: String,
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
}
