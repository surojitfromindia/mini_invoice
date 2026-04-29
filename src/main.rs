use tokio::net::TcpListener;

mod app;
mod config;
mod db;
mod routes;
mod app_state;
mod entity;
mod service_cotext;
mod request_context;
mod service;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = app::create_app().await?;
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
