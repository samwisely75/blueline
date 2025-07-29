//! Configuration constants and utilities for blueline
//!
//! This module contains blueline-specific configuration constants and utilities,
//! separate from the bluenote library's configuration.

/// Default profile file path for blueline
pub const DEFAULT_PROFILE_PATH: &str = "~/.blueline/profile";

/// Environment variable name for overriding the profile path
pub const PROFILE_PATH_ENV_VAR: &str = "BLUELINE_PROFILE_PATH";

/// Get the profile file path, checking environment variable first, then falling back to default
pub fn get_profile_path() -> String {
    std::env::var_os(PROFILE_PATH_ENV_VAR)
        .and_then(|val| val.into_string().ok())
        .unwrap_or_else(|| DEFAULT_PROFILE_PATH.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profile_path() {
        assert_eq!(DEFAULT_PROFILE_PATH, "~/.blueline/profile");
    }

    #[test]
    fn test_env_var_name() {
        assert_eq!(PROFILE_PATH_ENV_VAR, "BLUELINE_PROFILE_PATH");
    }

    #[test]
    fn test_get_profile_path_default() {
        // Save current env var state
        let original = std::env::var_os(PROFILE_PATH_ENV_VAR);

        // Remove env var if set
        std::env::remove_var(PROFILE_PATH_ENV_VAR);
        assert_eq!(get_profile_path(), DEFAULT_PROFILE_PATH);

        // Restore original state
        if let Some(val) = original {
            std::env::set_var(PROFILE_PATH_ENV_VAR, val);
        }
    }

    #[test]
    fn test_get_profile_path_env_override() {
        // Save current env var state
        let original = std::env::var_os(PROFILE_PATH_ENV_VAR);

        let test_path = "/custom/profile/path";
        std::env::set_var(PROFILE_PATH_ENV_VAR, test_path);
        assert_eq!(get_profile_path(), test_path);

        // Restore original state
        match original {
            Some(val) => std::env::set_var(PROFILE_PATH_ENV_VAR, val),
            None => std::env::remove_var(PROFILE_PATH_ENV_VAR),
        }
    }
}
