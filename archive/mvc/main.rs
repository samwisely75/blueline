mod cmd_args;
mod repl;

use std::env;

use bluenote::{
    get_blank_profile, HttpConnectionProfile, IniProfile, IniProfileStore, Result,
    DEFAULT_INI_FILE_PATH,
};
use cmd_args::CommandLineArgs;
// use old_repl::VimRepl;
use repl::{controller::ReplController, view::create_default_view_manager};
use tracing_subscriber::{fmt::time::ChronoLocal, EnvFilter};

// tokio is primarily for async I/O operations in the bluenote HTTP client
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

    tracing::debug!(
        "DEFAULT_INI_FILE_PATH constant: '{}'",
        DEFAULT_INI_FILE_PATH
    );
    let ini_store = IniProfileStore::new(DEFAULT_INI_FILE_PATH);

    tracing::debug!(
        "Loading profile '{}' from '{}'",
        profile_name,
        DEFAULT_INI_FILE_PATH
    );
    let profile_result = ini_store.get_profile(profile_name)?;

    let profile = match profile_result {
        Some(p) => {
            tracing::debug!("Profile loaded successfully, server: {:?}", p.server());
            p
        }
        None => {
            tracing::debug!("Profile '{}' not found, using blank profile", profile_name);
            get_blank_profile()
        }
    };

    tracing::debug!("INI profile: {:?}", profile);

    // run the REPL
    run_repl(profile, cmd_args.verbose()).await

    //    // Use the existing REPL (default)
    //    let mut repl = VimRepl::new(profile, cmd_args.verbose())?;
    //    repl.run().await
}

/// Create and run the MVC-based REPL
async fn run_repl(profile: IniProfile, verbose: bool) -> Result<()> {
    let view_manager = create_default_view_manager();
    let mut controller = ReplController::new(profile, verbose, view_manager)?;
    controller.run().await
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_tracing_subscriber_should_initialize_logging_without_panic() {
        // Test that the function doesn't panic when called
        // Note: We can't test the actual logging setup easily without side effects
        // but we can ensure the function completes without errors
        init_tracing_subscriber();
    }

    #[test]
    fn run_repl_function_should_exist_and_be_callable() {
        // This test ensures the run_repl function signature is correct
        // We create a blank profile for testing
        let profile = bluenote::ini::get_blank_profile();

        // Test that we can create the function call (compilation test)
        let _future = run_repl(profile, false);

        // We don't actually execute it to avoid terminal initialization issues in tests
        // The fact that this compiles means the function signature is correct
    }
}
