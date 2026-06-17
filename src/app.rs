use crate::api;
use crate::app_state::AppState;
use crate::config;
use crate::config::tracing::init_tracing;
use crate::db::connection::{init_read_replica_db, init_write_replica_db};
use crate::mcp;
use axum::Router;
use axum::http::Method;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderName};
use axum::routing::get;
use tower_http::cors::{Any, CorsLayer};
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
        settings,
    };

    info!("Server started");
    let app = Router::new()
        .route("/health", get(check_health))
        .merge(api::routes::create_routes())
        .merge(mcp::routes(app_state.clone()))
        .layer(cors_layer())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);
    Ok(app)
}

fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            ACCEPT,
            AUTHORIZATION,
            CONTENT_TYPE,
            HeaderName::from_static("timezone"),
            HeaderName::from_static("x-request-timezone"),
            HeaderName::from_static("x-timezone"),
        ])
}

async fn check_health() -> &'static str {
    "I am fine!"
}
