use axum::Router;
use crate::app_state::AppState;

pub mod user_routes;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/api/v1/users", user_routes::routes())

}