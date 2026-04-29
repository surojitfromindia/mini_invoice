use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
    pub primary_read_replica: DatabaseConnection,
    pub primary_write_replica: DatabaseConnection,
}