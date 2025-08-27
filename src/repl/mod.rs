//! # MVVM Architecture Implementation
//!
//! This module contains the clean MVVM implementation for BlueLine.
//! All components are designed with clear separation of concerns and testability.

pub mod commands;
pub mod controllers;
pub mod io;
pub mod models;
pub mod services;
pub mod text;
pub mod unified_commands;
pub mod view_models;
pub mod views;

// Re-export core types
pub use controllers::AppController;
pub use view_models::*;
pub use views::*;

// Re-export event types from their new locations
pub use io::event_source::EventSource;
pub use io::terminal_event_source::TerminalEventSource;
pub use models::events::{EventBus, InputEvent, ModelEvent, SimpleEventBus, ViewEvent};
pub use models::pane_state::{EditorMode, Pane, PaneCapabilities};
pub use models::{LogicalPosition, LogicalRange};

// Re-export specific items from commands to avoid conflicts
pub use commands::{Command, CommandContext, CommandEvent, CommandRegistry, ViewModelSnapshot};

// Re-export specific items from models to avoid conflicts
pub use models::{BufferModel, HttpHeaders, RequestModel, ResponseModel};

// Re-export geometry types from models
pub use models::{Dimensions, Position};
