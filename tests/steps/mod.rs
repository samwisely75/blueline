//! Step definitions for Cucumber tests
//!
//! This module contains all the step definitions for Blueline integration tests.
//!
//! Steps are organized by feature domain for better maintainability:
//! - `application` - App lifecycle and setup
//! - `command_line` - Command mode and ex commands
//! - `http` - HTTP request/response operations
//! - `modes` - Mode transitions and verification
//! - `navigation` - Cursor movement and navigation
//! - `text_manipulation` - Text input and editing
//! - `visual_mode` - Visual mode operations
//! - `terminal` - Terminal state and rendering
//! - `window` - Window/pane management
//! - `text_advanced` - Advanced text operations (undo/redo, copy/paste)

pub mod application;
pub mod command_line;
pub mod http;
pub mod modes;
pub mod navigation;
pub mod terminal;
pub mod text_advanced;
pub mod text_manipulation;
pub mod visual_mode;
pub mod window;

// Re-export all step functions
