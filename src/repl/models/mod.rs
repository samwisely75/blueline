//! # Models Module
//!
//! Re-exports all model implementations organized by category.
//! This module maintains the same public API while organizing models
//! into logical groups for better maintainability.

// Import model modules
pub mod buffer_char;
pub mod buffer_model;
pub mod display_cache;
pub mod display_char;
pub mod display_line;
pub mod request_model;
pub mod response_model;
pub mod status_line;

// Re-export all models for easy access
pub use buffer_char::{BufferChar, BufferLine, CharacterBuffer};
pub use buffer_model::{BufferContent, BufferModel};
pub use display_cache::{build_display_cache, DisplayCache, DisplayPosition};
pub use display_char::DisplayChar;
pub use display_line::DisplayLine;
pub use request_model::{HttpHeaders, RequestModel};
pub use response_model::ResponseModel;
pub use status_line::{HttpStatus, StatusLine};

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
