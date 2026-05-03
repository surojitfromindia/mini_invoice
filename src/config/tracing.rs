use tracing_subscriber::{fmt, EnvFilter};

pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("debug")); // same as your old DEBUG

    fmt()
        .with_env_filter(env_filter)
        .with_file(true)          // add file
        .with_line_number(true)   // add line
        .init();
}