use axum::Router;

pub mod user_routes;

pub fn create_routes() -> Router {
    Router::new()
        .nest("/api/v1/users", user_routes::routes())

}