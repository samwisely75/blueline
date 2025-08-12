//! Configuration constants and utilities for blueline
//!
//! This module contains blueline-specific configuration constants and utilities,
//! separate from the bluenote library's configuration.

use crate::cmd_args::CommandLineArgs;
use std::fs;
use std::path::PathBuf;

/// Default profile file path for blueline
pub const DEFAULT_PROFILE_PATH: &str = "~/.blueline/profile";

/// Environment variable name for overriding the profile path
pub const PROFILE_PATH_ENV_VAR: &str = "BLUELINE_PROFILE_PATH";

/// Default config file path for blueline settings
pub const DEFAULT_CONFIG_PATH: &str = "~/.blueline/config";

/// Environment variable name for overriding the config path
pub const CONFIG_PATH_ENV_VAR: &str = "BLUELINE_CONFIG_PATH";

/// Unified application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Profile name to use for HTTP connections
    profile_name: String,
    /// Path to the profile file
    profile_path: String,
}

impl AppConfig {
    /// Create AppConfig from command line arguments
    pub fn from_args(cmd_args: CommandLineArgs) -> Self {
        Self {
            profile_name: cmd_args.profile().to_string(),
            profile_path: get_profile_path(),
        }
    }

    /// Create AppConfig with explicit values (useful for testing)
    pub fn new(profile_name: String, profile_path: String) -> Self {
        Self {
            profile_name,
            profile_path,
        }
    }

    /// Get the profile name
    pub fn profile_name(&self) -> &str {
        &self.profile_name
    }

    /// Get the profile path
    pub fn profile_path(&self) -> &str {
        &self.profile_path
    }
}

/// Get the profile file path, checking environment variable first, then falling back to default
pub fn get_profile_path() -> String {
    std::env::var_os(PROFILE_PATH_ENV_VAR)
        .and_then(|val| val.into_string().ok())
        .unwrap_or_else(|| DEFAULT_PROFILE_PATH.to_string())
}

/// Get the config file path, checking environment variable first, then falling back to default
pub fn get_config_path() -> String {
    std::env::var_os(CONFIG_PATH_ENV_VAR)
        .and_then(|val| val.into_string().ok())
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string())
}

/// Load configuration commands from the config file
/// Returns a vector of ex commands to execute, or an empty vector if file doesn't exist
pub fn load_config_commands() -> Vec<String> {
    let config_path = get_config_path();
    let expanded = shellexpand::tilde(&config_path);
    let expanded_path = PathBuf::from(expanded.as_ref());

    tracing::debug!("Loading config from: {:?}", expanded_path);

    match fs::read_to_string(&expanded_path) {
        Ok(content) => {
            let commands: Vec<String> = content
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
                .map(|line| line.trim().to_string())
                .collect();

            tracing::info!(
                "Loaded {} config commands from {:?}",
                commands.len(),
                expanded_path
            );
            commands
        }
        Err(e) => {
            tracing::debug!(
                "Config file not found or not readable: {:?} - {}",
                expanded_path,
                e
            );
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_default_profile_path() {
        assert_eq!(DEFAULT_PROFILE_PATH, "~/.blueline/profile");
    }

    #[test]
    fn test_env_var_name() {
        assert_eq!(PROFILE_PATH_ENV_VAR, "BLUELINE_PROFILE_PATH");
    }

    #[test]
    #[serial]
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
    #[serial]
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

    #[test]
    fn test_default_config_path() {
        assert_eq!(DEFAULT_CONFIG_PATH, "~/.blueline/config");
    }

    #[test]
    fn test_config_env_var_name() {
        assert_eq!(CONFIG_PATH_ENV_VAR, "BLUELINE_CONFIG_PATH");
    }

    #[test]
    #[serial]
    fn test_get_config_path_default() {
        // Save current env var state
        let original = std::env::var_os(CONFIG_PATH_ENV_VAR);

        // Remove env var if set
        std::env::remove_var(CONFIG_PATH_ENV_VAR);
        assert_eq!(get_config_path(), DEFAULT_CONFIG_PATH);

        // Restore original state
        if let Some(val) = original {
            std::env::set_var(CONFIG_PATH_ENV_VAR, val);
        }
    }

    #[test]
    #[serial]
    fn test_get_config_path_env_override() {
        // Save current env var state
        let original = std::env::var_os(CONFIG_PATH_ENV_VAR);

        let test_path = "/custom/config/path";
        std::env::set_var(CONFIG_PATH_ENV_VAR, test_path);
        assert_eq!(get_config_path(), test_path);

        // Restore original state
        match original {
            Some(val) => std::env::set_var(CONFIG_PATH_ENV_VAR, val),
            None => std::env::remove_var(CONFIG_PATH_ENV_VAR),
        }
    }

    #[test]
    fn test_shellexpand_tilde() {
        // Test that shellexpand properly expands tilde
        let expanded = shellexpand::tilde("~/test");
        assert!(expanded.starts_with("/") || expanded.starts_with("C:\\"));

        let expanded_home = shellexpand::tilde("~");
        assert!(expanded_home.starts_with("/") || expanded_home.starts_with("C:\\"));

        // Test non-tilde paths remain unchanged
        let absolute = shellexpand::tilde("/absolute/path");
        assert_eq!(absolute.as_ref(), "/absolute/path");

        let relative = shellexpand::tilde("relative/path");
        assert_eq!(relative.as_ref(), "relative/path");
    }

    #[test]
    fn test_load_config_commands_missing_file() {
        // Save current env var state
        let original = std::env::var_os(CONFIG_PATH_ENV_VAR);

        // Set to a non-existent file
        std::env::set_var(CONFIG_PATH_ENV_VAR, "/tmp/nonexistent_blueline_config_test");
        let commands = load_config_commands();
        assert_eq!(commands.len(), 0);

        // Restore original state
        match original {
            Some(val) => std::env::set_var(CONFIG_PATH_ENV_VAR, val),
            None => std::env::remove_var(CONFIG_PATH_ENV_VAR),
        }
    }
}
