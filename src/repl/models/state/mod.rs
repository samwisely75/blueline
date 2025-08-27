//! State-related models for application state management

pub mod app_state;
pub mod geometry;
pub mod logical_position;
pub mod pane_state;
pub mod selection;
pub mod yank_buffer;

// Re-export commonly used types
pub use app_state::AppState;
pub use geometry::{Dimensions, Position};
pub use logical_position::{LogicalPosition, LogicalRange};
pub use pane_state::PaneState;
pub use selection::Selection;
pub use yank_buffer::{ClipboardYankBuffer, MemoryYankBuffer, YankBuffer, YankEntry};
