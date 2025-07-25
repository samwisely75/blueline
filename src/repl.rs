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

pub mod commands;
pub mod controller;
pub mod model;
pub mod view;
pub mod view_trait;

// Re-export main types for convenience
pub use commands::Command;
pub use controller::ReplController;
pub use model::{AppState, RequestBuffer, ResponseBuffer};
pub use view::{RenderObserver, ViewManager};
pub use view_trait::ViewRenderer;

// Legacy types for compatibility during transition
pub use model::{EditorMode, Pane, VisualSelection};
