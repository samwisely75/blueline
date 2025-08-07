//! # Views Module
//!
//! Contains all view-related components for rendering the terminal interface.

pub mod ansi_escape_codes;
pub mod terminal_renderer;

// Re-export main types for convenience
pub use terminal_renderer::{TerminalRenderer, ViewRenderer};
