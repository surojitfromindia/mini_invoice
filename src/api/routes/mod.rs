use crate::app_state::AppState;
use axum::Router;

mod auth_routes;
mod branch_routes;
mod organization_routes;
mod staff_role_routes;
mod staff_routes;
mod user_routes;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/api/v1/auth", auth_routes::routes())
        .nest("/api/v1/branch", branch_routes::routes())
        .nest("/api/v1/staff_role", staff_role_routes::routes())
        .nest("/api/v1/user_account", user_routes::routes())
        .nest("/api/v1/staff", staff_routes::routes())
        .nest("/api/v1/organization", organization_routes::routes())
}
