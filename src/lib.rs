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

pub mod mvvm;

// Re-export main types for easy access
pub use mvvm::*;
