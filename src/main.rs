use tokio::net::TcpListener;

mod api;
mod app;
mod app_state;
mod auth;
mod config;
mod db;
mod entity;
mod errors;
mod mcp;
mod resolver;
mod service;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = app::create_app().await?;
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
