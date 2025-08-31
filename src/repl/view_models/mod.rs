//! # ViewModel Module
//!

pub mod app_view_model;
pub mod post_command_actions;

pub use crate::repl::models::app_state::AppState;
pub use crate::repl::models::app_state::AppState as ViewModel;
pub use crate::repl::models::app_state::DisplayLineData;
pub use app_view_model::AppViewModel;
pub use post_command_actions::{InputEvent, PostCommandAction};
