//! Display-related models for rendering and visual representation

pub mod display_cache;
pub mod display_char;
pub mod display_line;
pub mod screen_buffer;
pub mod status_line;

// Re-export commonly used types
pub use display_cache::DisplayCache;
pub use display_char::DisplayChar;
pub use display_line::DisplayLine;
pub use screen_buffer::{BufferCell, ScreenBuffer};
pub use status_line::StatusLine;