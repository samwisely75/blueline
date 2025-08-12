//! # BlueLine Main Entry Point
//!
//! Clean MVVM HTTP client with vim-style interface.

use anyhow::Result;
use blueline::{
    cmd_args::CommandLineArgs,
    config::AppConfig,
    repl::io::{TerminalEventStream, TerminalRenderStream},
    AppController,
};
use std::env;
use tracing_subscriber::{fmt::time::ChronoLocal, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing_subscriber();

    let cmd_args = CommandLineArgs::parse();
    let config = AppConfig::from_args(cmd_args);

    // Explicit dependency injection - clear what implementations are being used
    let mut app = AppController::with_io_streams(
        config,
        TerminalEventStream::new(),
        TerminalRenderStream::new(),
    )?;

    app.run().await?;
    Ok(())
}

fn init_tracing_subscriber() {
    // Check if we should use file logging
    let log_file = env::var_os("BLUELINE_LOG_FILE").and_then(|s| s.into_string().ok());

    let env_filter = EnvFilter::try_from_env("BLUELINE_LOG_LEVEL")
        .or_else(|_| EnvFilter::try_from_env("RUST_LOG"))
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
        // In REPL mode, minimize logging to prevent any potential background scrolling
        // Use the most restrictive filter and file output only
        let minimal_filter = EnvFilter::try_from_env("BLUELINE_LOG_LEVEL")
            .or_else(|_| EnvFilter::try_from_env("RUST_LOG"))
            .unwrap_or_else(|_| EnvFilter::new("off")) // Default to no logging
            .add_directive("blueline=warn".parse().unwrap()) // Only warnings and errors from our code
            .add_directive("reqwest=off".parse().unwrap())
            .add_directive("hyper=off".parse().unwrap())
            .add_directive("tokio=off".parse().unwrap())
            .add_directive("tracing=off".parse().unwrap())
            .add_directive("tracing_subscriber=off".parse().unwrap())
            .add_directive("tower_http=off".parse().unwrap())
            .add_directive("tower=off".parse().unwrap())
            .add_directive("tokio_util=off".parse().unwrap())
            .add_directive("tokio_rustls=off".parse().unwrap())
            .add_directive("rustls=off".parse().unwrap())
            .add_directive("rustls_pemfile=off".parse().unwrap())
            .add_directive("native_tls=off".parse().unwrap())
            .add_directive("tokio_stream=off".parse().unwrap());

        let file_appender = tracing_appender::rolling::never(".", "blueline.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::fmt()
            .with_env_filter(minimal_filter)
            .with_writer(non_blocking)
            .with_timer(ChronoLocal::rfc_3339())
            .init();

        // Leak the guard to ensure logs are flushed on exit
        Box::leak(Box::new(_guard));
    }
}
