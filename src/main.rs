mod cmd;
mod repl;

use bluenote::{get_blank_profile, IniProfileStore, Result, DEFAULT_INI_FILE_PATH};
use cmd::CommandLineArgs;
use repl::VimRepl;
use tracing_subscriber::{fmt::time::ChronoLocal, EnvFilter};

#[tracing::instrument]
#[tokio::main]
async fn main() -> Result<()> {
    init_tracing_subscriber();

    // Load command line arguments - only profile and verbose are supported now
    let cmd_args = CommandLineArgs::parse();

    // Load profile from INI file by name specified in --profile argument
    // (default to "default")
    // If the profile is not found, then use a blank profile.
    // Uses bluenote's default profile path
    let profile_name = cmd_args.profile();
    let ini_store = IniProfileStore::new(DEFAULT_INI_FILE_PATH);
    let profile = ini_store
        .get_profile(profile_name)?
        .unwrap_or(get_blank_profile());
    tracing::debug!("INI profile: {:?}", profile);

    // Create and run the VIM-like REPL - this is now the only mode
    let mut repl = VimRepl::new(profile, cmd_args.verbose())?;
    repl.run().await
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
            .add_directive("tokio_util=warn".parse().unwrap())
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
