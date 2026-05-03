use crate::config::database::db_options;
use sea_orm::{Database, DatabaseConnection};

pub async fn init_read_replica_db(db_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    let db_connection = Database::connect(db_options(db_url)).await?;
    assert!(db_connection.ping().await.is_ok());
    Ok(db_connection)
}

pub async fn init_write_replica_db(db_url: &str) -> Result<DatabaseConnection, sea_orm::DbErr> {
    let db_connection = Database::connect(db_options(db_url)).await?;
    assert!(db_connection.ping().await.is_ok());
    // db_connection
    //     .get_schema_registry("smart_audit::entity::*")
    //     .sync(&db_connection)
    //     .await?;
    Ok(db_connection)
}
