//! # MVVM Architecture Implementation
//!
//! This module contains the clean MVVM implementation for BlueLine.
//! All components are designed with clear separation of concerns and testability.

pub mod io;
pub mod models;
pub mod services;
pub mod view_models;
pub mod views;

// Re-export core types
pub use view_models::AppViewModel;
pub use views::*;

// Re-export event types from their new locations
pub use io::event_source::EventSource;
pub use io::terminal_event_source::TerminalEventSource;
pub use models::events::{EventBus, ModelEvent, SimpleEventBus};
pub use models::pane_state::{EditorMode, Pane, PaneCapabilities};
pub use models::{LogicalPosition, LogicalRange};
pub use view_models::{InputEvent, PostCommandAction};

// Re-export specific items from models to avoid conflicts
pub use models::{BufferModel, HttpHeaders, RequestModel, ResponseModel};

// Re-export geometry types from models
pub use models::{Dimensions, Position};
