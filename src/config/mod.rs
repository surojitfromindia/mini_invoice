pub mod settings;
pub mod database;

use config::{Config, Environment};
use settings::Settings;

pub fn load() -> Result<Settings, config::ConfigError> {
    dotenvy::dotenv().ok();
    Config::builder()
        .add_source(Environment::default())
        .build()?
        .try_deserialize()
}