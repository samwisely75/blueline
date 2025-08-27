//! State-related models for application state management

pub mod geometry;
pub mod logical_position;
pub mod selection;
pub mod yank_buffer;

// Re-export commonly used types
pub use geometry::{Dimensions, Position};
pub use logical_position::{LogicalPosition, LogicalRange};
pub use selection::Selection;
pub use yank_buffer::{ClipboardYankBuffer, MemoryYankBuffer, YankBuffer, YankEntry};