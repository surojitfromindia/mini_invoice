mod handlers;
mod result;
mod server;

use crate::app_state::AppState;
use axum::Router;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};

use self::server::MiniInvoiceMcpServer;

pub fn routes(app_state: AppState) -> Router<AppState> {
    let service: StreamableHttpService<MiniInvoiceMcpServer, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(MiniInvoiceMcpServer::new(app_state.clone())),
            Default::default(),
            StreamableHttpServerConfig::default(),
        );

    Router::new().nest_service("/mcp", service)
}
