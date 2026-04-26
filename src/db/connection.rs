use sea_orm::{Database, DatabaseConnection, ConnectOptions};
use std::time::Duration;

pub async fn init_db() -> Result<DatabaseConnection, sea_orm::DbErr> {
    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let mut opt = ConnectOptions::new(db_url);
    opt.max_connections(10)
        .min_connections(2)
        .connect_timeout(Duration::from_secs(5))
        .sqlx_logging(false);

    Database::connect(opt).await
}