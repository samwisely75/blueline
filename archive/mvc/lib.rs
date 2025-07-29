//! # Blueline HTTP Client Library
//!
//! This library provides the core components of the blueline HTTP client,
//! including the REPL interface, MVC architecture, and command processing.

pub mod repl;

// Re-export main types for convenience
pub use repl::{
    commands::Command,
    controller::ReplController,
    model::{AppState, EditorMode, Pane, RequestBuffer, ResponseBuffer},
    view::{create_default_view_manager, ViewManager},
    view_trait::ViewRenderer,
};
