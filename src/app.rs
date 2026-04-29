use axum::Router;
use crate::db::connection::init_db;
use crate::config;

pub async fn create_app() -> anyhow::Result<Router> {
    let settings = config::load()?;
    let _db = init_db(&settings.database_url).await?;
    println!("Server started");
    let app = Router::new()
        .merge(crate::routes::create_routes());
    Ok(app)
}