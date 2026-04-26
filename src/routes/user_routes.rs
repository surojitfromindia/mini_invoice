use axum::Router;
use sea_orm::DatabaseConnection;

pub fn routes() ->Router<DatabaseConnection> {

    Router::new()
        .route("/")

}