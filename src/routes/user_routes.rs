use axum::Router;
use axum::routing::get;
use crate::app_state::AppState;

pub fn routes() ->Router<AppState> {
    Router::new()
        .route("/", get(basic_handler))

}


async fn basic_handler() -> &'static str {
    "Hello, World!"
}