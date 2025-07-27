//! # MVVM Architecture Implementation
//!
//! This module contains the clean MVVM implementation for BlueLine.
//! All components are designed with clear separation of concerns and testability.

pub mod commands;
pub mod controller;
pub mod events;
pub mod http;
pub mod models;
pub mod view_models;
pub mod views;

// Re-export core types
pub use controller::AppController;
pub use events::*;
pub use http::*;
pub use view_models::*;
pub use views::*;

// Re-export specific items from commands to avoid conflicts
pub use commands::{Command, CommandContext, CommandEvent, CommandRegistry, ViewModelSnapshot};

// Re-export specific items from models to avoid conflicts
pub use models::{BufferModel, EditorModel, HttpHeaders, RequestModel, ResponseModel};
