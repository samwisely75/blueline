//! Buffer-related models for text storage and manipulation

pub mod buffer_char;
pub mod buffer_model;
pub mod request_model;
pub mod response_model;
pub mod yank_buffer;

// Re-export commonly used types
pub use buffer_char::{BufferChar, BufferLine, CharacterBuffer};
pub use buffer_model::{BufferContent, BufferModel};
pub use request_model::RequestModel;
pub use response_model::ResponseModel;
pub use yank_buffer::{ClipboardYankBuffer, MemoryYankBuffer, YankBuffer, YankEntry, YankType};
