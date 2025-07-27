//! # BlueLine Main Entry Point
//!
//! Clean MVVM HTTP client with vim-style interface.

use anyhow::Result;
use blueline::{cmd_args::CommandLineArgs, AppController};
use std::env;
use tracing_subscriber::{fmt::time::ChronoLocal, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing_subscriber();

    let cmd_args = CommandLineArgs::parse();
    let mut app = AppController::new(cmd_args)?;
    app.run().await?;
    Ok(())
}

fn init_tracing_subscriber() {
    // Check if we should use file logging
    let log_file = env::var_os("BLUELINE_LOG_FILE").and_then(|s| s.into_string().ok());

    let env_filter = EnvFilter::try_from_env("BLUELINE_LOG_LEVEL")
        .unwrap_or_else(|_| EnvFilter::new("info"))
        .add_directive("reqwest=warn".parse().unwrap())
        .add_directive("hyper=warn".parse().unwrap())
        .add_directive("tokio=warn".parse().unwrap())
        .add_directive("tracing=warn".parse().unwrap())
        .add_directive("tracing_subscriber=warn".parse().unwrap())
        .add_directive("tower_http=warn".parse().unwrap())
        .add_directive("tower=warn".parse().unwrap())
        .add_directive("tokio_util=warn".parse().unwrap())
        .add_directive("tokio_rustls=warn".parse().unwrap())
        .add_directive("rustls=warn".parse().unwrap())
        .add_directive("rustls_pemfile=warn".parse().unwrap())
        .add_directive("native_tls=warn".parse().unwrap())
        .add_directive("tokio_stream=warn".parse().unwrap())
        .add_directive("tokio_io=warn".parse().unwrap())
        .add_directive("tokio_timer=warn".parse().unwrap())
        .add_directive("tokio_sync=warn".parse().unwrap())
        .add_directive("tokio_task=warn".parse().unwrap())
        .add_directive("tokio_reactor=warn".parse().unwrap());

    if let Some(log_file_path) = log_file {
        // Use file logging
        let file_appender = tracing_appender::rolling::never(".", log_file_path);
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(non_blocking)
            .with_timer(ChronoLocal::rfc_3339())
            .init();

        // Leak the guard to ensure logs are flushed on exit
        Box::leak(Box::new(_guard));
    } else {
        // In REPL mode, avoid stderr to prevent background scrolling
        // Default to file logging to prevent terminal interference
        let file_appender = tracing_appender::rolling::never(".", "blueline.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(non_blocking)
            .with_timer(ChronoLocal::rfc_3339())
            .init();

        // Leak the guard to ensure logs are flushed on exit
        Box::leak(Box::new(_guard));
    }
}
