use axum::Router;
use crate::db::connection::init_db;

pub async fn create_app() -> anyhow::Result<Router> {
    let db = init_db().await?;

    let app = Router::new()
        .merge(crate::routes::user_routes::routes())
        .with_state(db);

    Ok(app)
}