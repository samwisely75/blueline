//! # MVVM Architecture Implementation
//!
//! This module contains the clean MVVM implementation for BlueLine.
//! All components are designed with clear separation of concerns and testability.

pub mod commands;
pub mod controller;
pub mod events;
pub mod models;
pub mod view_models;
pub mod views;

// Re-export core types
pub use commands::*;
pub use controller::AppController;
pub use events::*;
pub use models::*;
pub use view_models::*;
pub use views::*;
