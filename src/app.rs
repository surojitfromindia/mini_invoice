use crate::api;
use crate::app_state::AppState;
use crate::config;
use crate::config::tracing::init_tracing;
use crate::db::connection::{init_read_replica_db, init_write_replica_db};
use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;
use tracing::info;

pub async fn create_app() -> anyhow::Result<Router> {
    let settings = config::load()?;

    init_tracing();

    let primary_read_replica = init_read_replica_db(&settings.database_url).await?;
    let primary_write_replica = init_write_replica_db(&settings.database_url).await?;

    let app_state = AppState {
        primary_read_replica,
        primary_write_replica,
    };

    info!("Server started");
    let app = Router::new()
        .route("/health", get(check_health))
        .merge(api::routes::create_routes())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);
    Ok(app)
}

async fn check_health()-> &'static str {
    "I am fine!"
}
