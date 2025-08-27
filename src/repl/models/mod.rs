//! # Models Module
//!
//! Re-exports all model implementations organized by category.
//! This module maintains the same public API while organizing models
//! into logical groups for better maintainability.

// Import model subdirectories
pub mod buffer;
pub mod display;
pub mod state;

// Re-export all models for easy access (maintaining backward compatibility)
pub use buffer::buffer_char::{BufferChar, BufferLine, CharacterBuffer};
pub use buffer::buffer_model::{BufferContent, BufferModel};
pub use buffer::request_model::{HttpHeaders, RequestModel};
pub use buffer::response_model::ResponseModel;

pub use display::display_cache::{build_display_cache, DisplayCache};
pub use display::display_char::DisplayChar;
pub use display::display_line::DisplayLine;
pub use display::screen_buffer::{BufferCell, ScreenBuffer};
pub use display::status_line::{HttpStatus, StatusLine};

pub use state::geometry::{Dimensions, Position};
pub use state::logical_position::{LogicalPosition, LogicalRange};
pub use state::selection::Selection;
pub use state::yank_buffer::{
    ClipboardYankBuffer, MemoryYankBuffer, YankBuffer, YankEntry, YankType,
};

// Re-export submodules for direct access (backward compatibility)
pub use buffer::buffer_char;
pub use buffer::buffer_model;
pub use buffer::request_model;
pub use buffer::response_model;

pub use display::display_cache;
pub use display::display_char;
pub use display::display_line;
pub use display::screen_buffer;
pub use display::status_line;

pub use state::geometry;
pub use state::logical_position;
pub use state::selection;
pub use state::yank_buffer;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::Pane;

    #[test]
    fn all_models_should_create_with_defaults() {
        let _buffer = BufferModel::new(Pane::Request);
        let _request = RequestModel::new();
        let _response = ResponseModel::new();

        // If we get here without panicking, all models can be created
    }

    #[test]
    fn buffer_model_should_start_with_empty_content() {
        let buffer = BufferModel::new(Pane::Request);
        assert_eq!(buffer.content().get_text(), "");
    }
}
