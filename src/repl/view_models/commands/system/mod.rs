//! # System Commands Module
//!
//! This module contains commands related to system operations and application-level
//! functionality such as HTTP requests, profile management, and settings configuration.
//!
//! NOTE: Commands self-register via inventory system - adding new command files
//! here won't cause merge conflicts. Add modules alphabetically to prevent conflicts.

// Auto-discover system command modules - add new ones alphabetically
pub mod app_terminate;
pub mod ex_fallback;
pub mod ex_quit;
pub mod ex_quit_test;
pub mod ex_set_clipboard;
pub mod ex_set_dcut;
pub mod ex_set_expandtab;
pub mod ex_set_number;
pub mod ex_set_tabstop;
pub mod ex_set_wrap;
pub mod ex_show_profile;
pub mod http;
pub mod setting_change;

// Re-export all command types using wildcards - no merge conflicts!
pub use app_terminate::*;
pub use ex_fallback::*;
pub use ex_quit::*;
pub use ex_set_clipboard::*;
pub use ex_set_dcut::*;
pub use ex_set_expandtab::*;
pub use ex_set_number::*;
pub use ex_set_tabstop::*;
pub use ex_set_wrap::*;
pub use ex_show_profile::*;
pub use http::*;
pub use setting_change::*;
