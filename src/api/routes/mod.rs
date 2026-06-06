use crate::app_state::AppState;
use aide::axum::ApiRouter;
use aide::openapi::{Info, OpenApi};
use aide::swagger::Swagger;
use axum::routing::get;
use axum::{Extension, Json, Router};

mod auth_routes;
mod branch_routes;
mod item_routes;
mod organization_routes;
mod staff_role_routes;
mod staff_routes;
mod user_routes;

pub fn create_routes() -> Router<AppState> {
    let mut api = OpenApi {
        info: Info {
            title: "Mini Invoice API".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let item_routes = ApiRouter::new()
        .nest("/api/v1/item", item_routes::routes())
        .finish_api(&mut api);

    Router::new()
        .nest("/api/v1/auth", auth_routes::routes())
        .nest("/api/v1/branch", branch_routes::routes())
        .merge(item_routes)
        .nest("/api/v1/staff_role", staff_role_routes::routes())
        .nest("/api/v1/user_account", user_routes::routes())
        .nest("/api/v1/staff", staff_routes::routes())
        .nest("/api/v1/organization", organization_routes::routes())
        .route("/api/docs/openapi.json", get(serve_openapi))
        .route(
            "/api/docs/swagger",
            Swagger::new("/api/docs/openapi.json").axum_route().into(),
        )
        .layer(Extension(api))
}

async fn serve_openapi(Extension(api): Extension<OpenApi>) -> Json<OpenApi> {
    Json(api)
}
