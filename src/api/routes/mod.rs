use axum::Router;
use crate::app_state::AppState;

mod user_routes;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/api/v1/user_account", user_routes::routes())

}