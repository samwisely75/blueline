//! # Yank Buffer Module
//!
//! Provides abstraction for text copying/pasting operations.
//! Supports both memory-based and system clipboard implementations.

use anyhow::Result;

/// Trait for yank buffer implementations
#[allow(dead_code)]
pub trait YankBuffer: Send {
    /// Store text in the yank buffer
    fn yank(&mut self, text: String) -> Result<()>;

    /// Retrieve text from the yank buffer
    fn paste(&self) -> Option<&str>;

    /// Clear the yank buffer
    fn clear(&mut self);

    /// Check if the yank buffer has content
    fn has_content(&self) -> bool;
}

/// Memory-based yank buffer implementation
#[derive(Debug, Default)]
pub struct MemoryYankBuffer {
    content: Option<String>,
}

impl MemoryYankBuffer {
    /// Create a new empty memory yank buffer
    pub fn new() -> Self {
        Self { content: None }
    }
}

impl YankBuffer for MemoryYankBuffer {
    fn yank(&mut self, text: String) -> Result<()> {
        tracing::debug!("Yanking {} characters to memory buffer", text.len());
        self.content = Some(text);
        Ok(())
    }

    fn paste(&self) -> Option<&str> {
        self.content.as_deref()
    }

    fn clear(&mut self) {
        self.content = None;
    }

    fn has_content(&self) -> bool {
        self.content.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_yank_buffer_should_store_and_retrieve_text() {
        let mut buffer = MemoryYankBuffer::new();

        // Initially empty
        assert!(!buffer.has_content());
        assert_eq!(buffer.paste(), None);

        // Yank some text
        buffer.yank("Hello, world!".to_string()).unwrap();
        assert!(buffer.has_content());
        assert_eq!(buffer.paste(), Some("Hello, world!"));

        // Replace with new text
        buffer.yank("New text".to_string()).unwrap();
        assert_eq!(buffer.paste(), Some("New text"));

        // Clear buffer
        buffer.clear();
        assert!(!buffer.has_content());
        assert_eq!(buffer.paste(), None);
    }
}
