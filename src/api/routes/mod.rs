use crate::app_state::AppState;
use axum::Router;

mod organization_routes;
mod user_routes;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/api/v1/user_account", user_routes::routes())
        .nest("/api/v1/organization", organization_routes::routes())
}
