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
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_env(format!(
                "{}_LOG_LEVEL",
                env!("CARGO_PKG_NAME").to_uppercase()
            ))
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
            .add_directive("tokio_reactor=warn".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .with_timer(ChronoLocal::rfc_3339())
        .init();
}
