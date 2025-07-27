//! Models module for MVVM architecture
//!
//! This module contains all the specialized models that handle different aspects
//! of the application state without any view concerns.

pub mod buffer_model;
pub mod editor_model;
pub mod request_model;
pub mod response_model;

// Re-export main types for convenience
pub use buffer_model::{BufferContent, BufferModel};
pub use editor_model::EditorModel;
pub use request_model::{HttpMethod, RequestModel};
pub use response_model::{ResponseModel, ResponseStatus, ResponseTiming};
