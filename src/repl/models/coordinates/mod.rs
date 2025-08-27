//! Coordinate-related models for positions and selections

pub mod geometry;
pub mod logical_position;
pub mod selection;

// Re-export commonly used types
pub use geometry::{Dimensions, Position};
pub use logical_position::{LogicalPosition, LogicalRange};
pub use selection::Selection;
