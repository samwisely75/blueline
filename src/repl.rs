//! # REPL Module - MVC Architecture
//!
//! This module implements a vim-style HTTP client REPL using MVC pattern:
//!
//! - **Model**: Buffer states (RequestBuffer, ResponseBuffer) and application state
//! - **View**: Terminal rendering components (panes, status bar, observers)  
//! - **Controller**: Command processors that handle key events and update models
//!
//! ## Architecture Overview
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │    Controller   │    │      Model      │    │      View       │
//! │                 │    │                 │    │                 │
//! │ • Command trait │────▶│ • Buffers      │────▶│ • Panes        │
//! │ • Key handlers  │    │ • App state     │    │ • Status bar    │
//! │ • Mode logic    │    │ • Cursor pos    │    │ • Observers     │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//! ```
//!

#![allow(dead_code)] // Allow unused code during refactoring
#![allow(unused_imports)] // Allow unused imports during refactoring
//! This separation eliminates the current tight coupling where buffers handle
//! both data storage AND key processing, improving maintainability and testability.

pub mod command;
pub mod controller;

// Command modules
pub mod commands {
    pub mod editing;
    pub mod movement;

    // Re-export commonly used commands
    pub use editing::*;
    pub use movement::*;
}

pub mod model;
pub mod new_repl;
pub mod view;

#[cfg(test)]
pub mod testing;

// Re-export main types for convenience
pub use command::Command;
pub use controller::ReplController;
pub use model::{AppState, RequestBuffer, ResponseBuffer};
pub use new_repl::run_new_repl;
pub use view::{RenderObserver, ViewManager};

// Legacy types for compatibility during transition
pub use model::{EditorMode, Pane, VisualSelection};
