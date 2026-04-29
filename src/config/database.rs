use sea_orm::ConnectOptions;
use std::time::Duration;

pub fn db_options(database_url: &str) -> ConnectOptions {
    let mut opt = ConnectOptions::new(database_url);

    opt.max_connections(10)
        .min_connections(2)
        .connect_timeout(Duration::from_secs(5))
        .sqlx_logging(false)
        .after_connect(|_conn| {
            Box::pin(async move {
                println!("DB connected");
                Ok(())
            })
        });
    opt
}