//! Organized step definitions for Cucumber tests
//!
//! This module reorganizes the step definitions into logical groups
//! to improve maintainability and readability.

pub mod assertions;
pub mod editing;
pub mod http;
pub mod mode;
pub mod navigation;
pub mod rendering;
pub mod setup;
pub mod visual;

// Re-export all step definitions so they're available when importing this module
pub use assertions::*;
pub use editing::*;
pub use http::*;
pub use mode::*;
pub use navigation::*;
pub use rendering::*;
pub use setup::*;
pub use visual::*;