//! # Utils Module
//!
//! Contains utility functions and helpers used across the REPL.

pub mod character_navigation;
pub mod http_parser;

// Re-export main functions for convenience
pub use character_navigation::*;
pub use http_parser::{create_default_profile, parse_request_from_text};
