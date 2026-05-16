use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use crate::app_state::AppState;
use crate::service::service_context::ServiceContext;

pub async fn service_context_middleware(
    State(app_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let ctx = ServiceContext::from_app_state(app_state);
    request.extensions_mut().insert(ctx);
    next.run(request).await
}