use crate::app_state::AppState;
use crate::config;
use crate::db::connection::{init_read_replica_db, init_write_replica_db};
use axum::Router;
use crate::config::tracing::init_tracing;

pub async fn create_app() -> anyhow::Result<Router> {
    let settings = config::load()?;

    init_tracing();

    let primary_read_replica = init_read_replica_db(&settings.database_url).await?;
    let primary_write_replica = init_write_replica_db(&settings.database_url).await?;

    let app_state = AppState {
        primary_read_replica,
        primary_write_replica,
    };

    println!("Server started");
    let app = Router::new()
        .merge(crate::routes::create_routes())
        .with_state(app_state);
    Ok(app)
}
