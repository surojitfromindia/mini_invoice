use crate::app_state::AppState;
use axum::Router;

mod user_routes;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/api/v1/user_account", user_routes::routes())

}