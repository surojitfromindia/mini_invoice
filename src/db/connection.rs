use sea_orm::{Database, DatabaseConnection, ConnectOptions};
use std::time::Duration;
use crate::config::database::db_options;

pub async fn init_db(db_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    Database::connect(db_options(db_url)).await
}