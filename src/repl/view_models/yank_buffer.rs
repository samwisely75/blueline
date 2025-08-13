//! # Yank Buffer Module
//!
//! Provides abstraction for text copying/pasting operations.
//! Supports both memory-based and system clipboard implementations.

use anyhow::Result;
use std::sync::{Arc, Mutex};

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

/// System clipboard-based yank buffer implementation
pub struct ClipboardYankBuffer {
    /// Cache for the last yanked text (needed for the &str return type)
    cached_content: Option<String>,
    /// The actual clipboard instance wrapped in Arc<Mutex> for thread safety
    clipboard: Arc<Mutex<arboard::Clipboard>>,
}

impl std::fmt::Debug for ClipboardYankBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClipboardYankBuffer")
            .field("cached_content", &self.cached_content)
            .field("clipboard", &"<system clipboard>")
            .finish()
    }
}

impl ClipboardYankBuffer {
    /// Create a new clipboard yank buffer
    pub fn new() -> Result<Self> {
        let clipboard = arboard::Clipboard::new()
            .map_err(|e| anyhow::anyhow!("Failed to access system clipboard: {}", e))?;

        Ok(Self {
            cached_content: None,
            clipboard: Arc::new(Mutex::new(clipboard)),
        })
    }

    /// Sync cached content with actual clipboard content
    /// This is needed because another application might have changed the clipboard
    #[allow(dead_code)]
    fn sync_from_clipboard(&mut self) {
        if let Ok(mut clipboard) = self.clipboard.lock() {
            if let Ok(text) = clipboard.get_text() {
                // Only update cache if clipboard has content
                if !text.is_empty() {
                    self.cached_content = Some(text);
                }
            }
        }
    }
}

impl YankBuffer for ClipboardYankBuffer {
    fn yank(&mut self, text: String) -> Result<()> {
        tracing::debug!("Yanking {} characters to system clipboard", text.len());

        // Update the cache first
        self.cached_content = Some(text.clone());

        // Then update the system clipboard
        let mut clipboard = self
            .clipboard
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock clipboard: {}", e))?;

        clipboard
            .set_text(text)
            .map_err(|e| anyhow::anyhow!("Failed to set clipboard text: {}", e))?;

        Ok(())
    }

    fn paste(&self) -> Option<&str> {
        // Return cached content
        // Note: This doesn't sync with system clipboard changes from other apps
        // because we need to return a reference, not an owned String
        self.cached_content.as_deref()
    }

    fn clear(&mut self) {
        self.cached_content = None;
        // Also clear the system clipboard
        if let Ok(mut clipboard) = self.clipboard.lock() {
            let _ = clipboard.clear();
        }
    }

    fn has_content(&self) -> bool {
        // Check cached content first
        if self.cached_content.is_some() {
            return true;
        }

        // Fall back to checking actual clipboard
        if let Ok(mut clipboard) = self.clipboard.lock() {
            if let Ok(text) = clipboard.get_text() {
                return !text.is_empty();
            }
        }

        false
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
