//! # BlueLine - Terminal HTTP Client with Vim-like Interface
//!
//! A modern REPL for testing HTTP APIs with vim-style key bindings.
//! Built with clean MVVM architecture for maintainability and testability.
//!
//! ## Architecture
//!
//! This application follows the Model-View-ViewModel (MVVM) pattern:
//!
//! ```text
//! ┌─────────────┐    Events    ┌──────────────┐    Updates   ┌─────────┐
//! │    View     │◄─────────────│  ViewModel   │◄─────────────│ Models  │
//! │             │              │              │              │         │
//! │ - Terminal  │              │ - Business   │              │ - Data  │
//! │ - Rendering │              │   Logic      │              │ - State │
//! │ - Input     │              │ - Coordination│              │         │
//! └─────────────┘              └──────────────┘              └─────────┘
//!                                      ▲
//!                                      │ Commands
//!                                      ▼
//!                               ┌──────────────┐
//!                               │  Controller  │
//!                               │              │
//!                               │ - Input      │
//!                               │   Mapping    │
//!                               │ - Event Loop │
//!                               └──────────────┘
//! ```

pub mod cmd_args;
pub mod config;
pub mod repl;

/// Macro for self-registering commands
///
/// Usage in command modules:
/// ```rust,ignore
/// # // This is a compile-time macro that requires inventory setup
/// register_command!(YankSelectionCommand, "YankSelectionCommand");
/// ```
#[macro_export]
macro_rules! register_command {
    ($command_type:ty, $name:literal) => {
        inventory::submit! {
            $crate::repl::view_models::commands::CommandEntry {
                name: $name,
                factory: || Box::new(<$command_type>::new()),
            }
        }
    };
}

// Re-export main types for easy access
pub use repl::*;
