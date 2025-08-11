//! # MVVM Architecture Implementation
//!
//! This module contains the clean MVVM implementation for BlueLine.
//! All components are designed with clear separation of concerns and testability.

pub mod commands;
pub mod controllers;
pub mod events;
pub mod geometry;
pub mod io;
pub mod models;
pub mod text;
pub mod utils;
pub mod view_models;
pub mod views;

// Re-export core types
pub use controllers::AppController;
pub use events::*;
pub use utils::*;
pub use view_models::*;
pub use views::*;

// Re-export specific items from commands to avoid conflicts
pub use commands::{Command, CommandContext, CommandEvent, CommandRegistry, ViewModelSnapshot};

// Re-export specific items from models to avoid conflicts
pub use models::{BufferModel, HttpHeaders, RequestModel, ResponseModel};

// Re-export geometry types
pub use geometry::{Dimensions, Position};
