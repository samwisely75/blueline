use std::ffi::OsString;

pub use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ClapArgs {
    /// Profile name
    /// Required. Profile name to use for the request. Default is 'default'.
    /// If the profile is not configured, the request will fail.
    #[clap(short = 'p', long, default_value = "default", help = "profile name")]
    profile: String,

    /// Verbose mode
    /// Optional. Print verbose messages.
    #[clap(
        short = 'v',
        long,
        help = "Print verbose message",
        default_value = "false"
    )]
    verbose: bool,
}

#[derive(Debug, Clone)]
pub struct CommandLineArgs {
    #[allow(dead_code)] // Used by profile() method
    profile: String,
    #[allow(dead_code)] // Used by verbose() method
    verbose: bool,
}

impl CommandLineArgs {
    #[allow(dead_code)]
    pub fn parse() -> Self {
        let args = ClapArgs::parse();
        Self {
            profile: args.profile,
            verbose: args.verbose,
        }
    }

    #[allow(dead_code)]
    pub fn parse_from<I, T>(itr: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OsString> + Clone,
    {
        let args = ClapArgs::parse_from(itr);
        Self {
            profile: args.profile,
            verbose: args.verbose,
        }
    }

    #[allow(dead_code)]
    pub fn profile(&self) -> &String {
        &self.profile
    }

    #[allow(dead_code)]
    pub fn verbose(&self) -> bool {
        self.verbose
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_args_profile_only() {
        let args = CommandLineArgs::parse_from(["program", "--profile", "test"]);
        assert_eq!(args.profile(), "test");
        assert!(!args.verbose());
    }

    #[test]
    fn test_parse_args_verbose() {
        let args = CommandLineArgs::parse_from(["program", "--verbose"]);
        assert_eq!(args.profile(), "default");
        assert!(args.verbose());
    }

    #[test]
    fn test_parse_args_short_flags() {
        let args = CommandLineArgs::parse_from(["program", "-p", "dev", "-v"]);
        assert_eq!(args.profile(), "dev");
        assert!(args.verbose());
    }

    #[test]
    fn test_default_values() {
        let args = CommandLineArgs::parse_from(["program"]);
        assert_eq!(args.profile(), "default");
        assert!(!args.verbose());
    }
}
