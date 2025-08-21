//! # Unified Command Pattern Infrastructure  
//!
//! This module implements the new unified Command Pattern architecture that
//! replaces the old command system. Commands contain both key binding logic
//! (is_relevant) and business logic (handle), emitting semantic ModelEvents.

// Core infrastructure
pub mod command;
pub mod events;
pub mod registry;

// Command implementations
pub mod yank;

// Re-export main types
pub use command::{Command, CommandContext};
pub use events::{ModelEvent, YankType};
pub use registry::UnifiedCommandRegistry;
