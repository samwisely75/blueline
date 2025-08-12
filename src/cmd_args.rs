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
}

#[derive(Debug, Clone)]
pub struct CommandLineArgs {
    #[allow(dead_code)] // Used by profile() method
    profile: String,
}

impl CommandLineArgs {
    #[allow(dead_code)]
    pub fn parse() -> Self {
        let args = ClapArgs::parse();
        Self {
            profile: args.profile,
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
        }
    }

    #[allow(dead_code)]
    pub fn profile(&self) -> &String {
        &self.profile
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_args_profile_only() {
        let args = CommandLineArgs::parse_from(["program", "--profile", "test"]);
        assert_eq!(args.profile(), "test");
    }

    #[test]
    fn test_parse_args_short_flags() {
        let args = CommandLineArgs::parse_from(["program", "-p", "dev"]);
        assert_eq!(args.profile(), "dev");
    }

    #[test]
    fn test_default_values() {
        let args = CommandLineArgs::parse_from(["program"]);
        assert_eq!(args.profile(), "default");
    }
}
