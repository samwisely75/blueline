mod cmd;
mod decoder;
mod http;
mod ini;
mod repl;
// mod stdio;  // Suspended - piped input disabled for security reasons until further notice
mod url;
mod utils;

use cmd::CommandLineArgs;
use ini::{get_blank_profile, IniProfileStore, DEFAULT_INI_FILE_PATH};
use repl::VimRepl;
use tracing_subscriber::{fmt::time::ChronoLocal, EnvFilter};
use utils::Result;

#[tracing::instrument]
#[tokio::main]
async fn main() -> Result<()> {
    init_tracing_subscriber();

    // Load command line arguments - only profile and verbose are supported now
    let cmd_args = CommandLineArgs::parse();

    // Load profile from INI file by name specified in --profile argument
    // (default to "default")
    // If the profile is not found, then use a blank profile.
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

/// Print connection profile details to stderr when verbose mode is enabled
/// This function displays host, port, scheme, certificates, authentication, headers, and proxy settings
#[tracing::instrument]
pub fn print_profile(profile: &impl http::HttpConnectionProfile) {
    if let Some(endpoint) = profile.server() {
        eprintln!("> connection:");
        eprintln!(">   host: {}", endpoint.host());
        eprintln!(
            ">   port: {}",
            endpoint
                .port()
                .map(|p| p.to_string())
                .unwrap_or("<none>".to_string())
        );
        eprintln!(">   scheme: {}", endpoint.scheme().unwrap());
        if endpoint.scheme().unwrap() == "https" {
            eprintln!(
                ">   ca-cert: {}",
                profile.ca_cert().unwrap_or(&"<none>".to_string())
            );
            eprintln!(
                ">   insecure: {}",
                profile
                    .insecure()
                    .map(|x| x.to_string())
                    .unwrap_or("<none>".to_string())
            );
        }
    } else {
        eprintln!("> connection: <none>");
    }
    if profile.user().is_some() {
        eprintln!(">   user: {}", profile.user().unwrap());
        eprintln!(
            ">   password: {}",
            profile.password().map(|_| "<provided>").unwrap_or("<none>")
        );
    }
    eprintln!(">   headers:");
    profile.headers().iter().for_each(|(name, value)| {
        eprintln!(">    {name}: {value}");
    });
    if profile.proxy().is_some() {
        eprintln!(">   proxy: {}", profile.proxy().unwrap());
    }
}

/// Print HTTP request details to stderr when verbose mode is enabled
/// This function displays the HTTP method, URL path, and request body (truncated if long)
#[tracing::instrument]
pub fn print_request(req: &impl http::HttpRequestArgs) {
    let url = req
        .url_path()
        .map(|u| u.to_string())
        .unwrap_or("<none>".to_string());
    eprintln!("> request:");
    eprintln!(">   method: {}", req.method().unwrap());
    eprintln!(">   path: {url}");
    eprintln!(
        ">   body: {}",
        req.body()
            .map(|b| if b.len() > 78 {
                format!("{}...", &b[0..75])
            } else {
                b.to_string()
            })
            .unwrap_or("<none>".to_string())
    );
}

/// Print HTTP response details to stderr when verbose mode is enabled
/// This function displays the response status code and all response headers
pub fn print_response(res: &http::HttpResponse) {
    eprintln!("> response:");
    eprintln!(">   status: {}", res.status());
    eprintln!(">   headers:");
    res.headers().iter().for_each(|(name, value)| {
        eprintln!(">     {}: {}", name, value.to_str().unwrap());
    });
}

/// Format connection profile details as a string for display in REPL response pane
/// Returns a formatted string showing host, port, scheme, certificates, authentication, headers, and proxy settings
pub fn format_profile(profile: &impl http::HttpConnectionProfile) -> String {
    let mut output = String::new();
    
    if let Some(endpoint) = profile.server() {
        output.push_str("> connection:\n");
        output.push_str(&format!(">   host: {}\n", endpoint.host()));
        output.push_str(&format!(
            ">   port: {}\n",
            endpoint
                .port()
                .map(|p| p.to_string())
                .unwrap_or("<none>".to_string())
        ));
        output.push_str(&format!(">   scheme: {}\n", endpoint.scheme().unwrap()));
        if endpoint.scheme().unwrap() == "https" {
            output.push_str(&format!(
                ">   ca-cert: {}\n",
                profile.ca_cert().unwrap_or(&"<none>".to_string())
            ));
            output.push_str(&format!(
                ">   insecure: {}\n",
                profile
                    .insecure()
                    .map(|x| x.to_string())
                    .unwrap_or("<none>".to_string())
            ));
        }
    } else {
        output.push_str("> connection: <none>\n");
    }
    
    if profile.user().is_some() {
        output.push_str(&format!(">   user: {}\n", profile.user().unwrap()));
        output.push_str(&format!(
            ">   password: {}\n",
            profile.password().map(|_| "<provided>").unwrap_or("<none>")
        ));
    }
    
    output.push_str(">   headers:\n");
    profile.headers().iter().for_each(|(name, value)| {
        output.push_str(&format!(">    {name}: {value}\n"));
    });
    
    if profile.proxy().is_some() {
        output.push_str(&format!(">   proxy: {}\n", profile.proxy().unwrap()));
    }
    
    output
}

/// Format HTTP request details as a string for display in REPL response pane
/// Returns a formatted string showing the HTTP method, URL path, and request body (truncated if long)
pub fn format_request(req: &impl http::HttpRequestArgs) -> String {
    let url = req
        .url_path()
        .map(|u| u.to_string())
        .unwrap_or("<none>".to_string());
        
    let mut output = String::new();
    output.push_str("> request:\n");
    output.push_str(&format!(">   method: {}\n", req.method().unwrap()));
    output.push_str(&format!(">   path: {url}\n"));
    output.push_str(&format!(
        ">   body: {}\n",
        req.body()
            .map(|b| if b.len() > 78 {
                format!("{}...", &b[0..75])
            } else {
                b.to_string()
            })
            .unwrap_or("<none>".to_string())
    ));
    
    output
}

/// Format HTTP response details as a string for display in REPL response pane
/// Returns a formatted string showing the response status code and all response headers
pub fn format_response(res: &http::HttpResponse) -> String {
    let mut output = String::new();
    output.push_str("> response:\n");
    output.push_str(&format!(">   status: {}\n", res.status()));
    output.push_str(">   headers:\n");
    res.headers().iter().for_each(|(name, value)| {
        output.push_str(&format!(">     {}: {}\n", name, value.to_str().unwrap()));
    });
    
    output
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
