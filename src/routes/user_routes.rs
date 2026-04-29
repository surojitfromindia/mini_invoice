use axum::Router;
use axum::routing::get;
use sea_orm::DatabaseConnection;

pub fn routes() ->Router {
    Router::new()
        .route("/", get(basic_handler))

}


async fn basic_handler() -> &'static str {
    "Hello, World!"
}