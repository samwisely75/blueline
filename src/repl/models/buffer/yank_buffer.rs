//! # Yank Buffer Module
//!
//! Provides abstraction for text copying/pasting operations.
//! Supports both memory-based and system clipboard implementations.
//! Enhanced with yank type metadata for proper block-wise operations.

use anyhow::Result;
use std::sync::{Arc, Mutex};

/// Type of yank operation, determining paste behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YankType {
    /// Character-wise selection (regular visual mode)
    Character,
    /// Line-wise selection (visual line mode)
    Line,
    /// Block-wise selection (visual block mode)
    Block,
}

/// Yank entry containing text and metadata
#[derive(Debug, Clone, PartialEq)]
pub struct YankEntry {
    /// The yanked text content
    pub text: String,
    /// Type of yank operation
    pub yank_type: YankType,
}

/// Trait for yank buffer implementations
#[allow(dead_code)]
pub trait YankBuffer: Send {
    /// Store text with type metadata in the yank buffer
    fn yank_with_type(&mut self, text: String, yank_type: YankType) -> Result<()>;

    /// Store text in the yank buffer (defaults to Character type for backward compatibility)
    fn yank(&mut self, text: String) -> Result<()> {
        self.yank_with_type(text, YankType::Character)
    }

    /// Retrieve yank entry from the yank buffer
    /// Takes &mut self to allow syncing with system clipboard
    fn paste_entry(&mut self) -> Option<YankEntry>;

    /// Retrieve text from the yank buffer (for backward compatibility)
    /// Takes &mut self to allow syncing with system clipboard
    fn paste(&mut self) -> Option<&str>;

    /// Clear the yank buffer
    fn clear(&mut self);

    /// Check if the yank buffer has content
    fn has_content(&self) -> bool;
}

/// Memory-based yank buffer implementation
#[derive(Debug, Default)]
pub struct MemoryYankBuffer {
    content: Option<YankEntry>,
}

impl MemoryYankBuffer {
    /// Create a new empty memory yank buffer
    pub fn new() -> Self {
        Self { content: None }
    }
}

impl YankBuffer for MemoryYankBuffer {
    fn yank_with_type(&mut self, text: String, yank_type: YankType) -> Result<()> {
        tracing::debug!(
            "Yanking {} characters to memory buffer (type: {:?})",
            text.len(),
            yank_type
        );
        self.content = Some(YankEntry { text, yank_type });
        Ok(())
    }

    fn paste_entry(&mut self) -> Option<YankEntry> {
        self.content.clone()
    }

    fn paste(&mut self) -> Option<&str> {
        self.content.as_ref().map(|entry| entry.text.as_str())
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
    /// Cache for the last yanked entry (needed for the reference return type)
    cached_content: Option<YankEntry>,
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
    /// Note: External clipboard changes default to Character type since we can't know the original type
    fn sync_from_clipboard(&mut self) {
        if let Ok(mut clipboard) = self.clipboard.lock() {
            if let Ok(clipboard_text) = clipboard.get_text() {
                // Only update if the clipboard text has actually changed (indicating external modification)
                // or if we have no cached content yet
                let should_update = match &self.cached_content {
                    Some(cached) => {
                        let text_changed = cached.text != clipboard_text;
                        if text_changed {
                            tracing::debug!(
                                "Clipboard text changed from '{}' to '{}'",
                                cached.text,
                                clipboard_text
                            );
                        } else {
                            tracing::debug!(
                                "Clipboard text unchanged, preserving type: {:?}",
                                cached.yank_type
                            );
                        }
                        text_changed
                    }
                    None => {
                        tracing::debug!("No cached content, syncing from clipboard");
                        true
                    }
                };

                if should_update {
                    // External change detected: update cache with Character type (since we can't know the original type)
                    tracing::debug!("Updating cache with Character type due to external change");
                    self.cached_content = Some(YankEntry {
                        text: clipboard_text,
                        yank_type: YankType::Character,
                    });
                }
                // If text matches our cache, preserve our existing metadata (including yank_type)
            }
        }
    }
}

impl YankBuffer for ClipboardYankBuffer {
    fn yank_with_type(&mut self, text: String, yank_type: YankType) -> Result<()> {
        tracing::debug!(
            "Yanking {} characters to system clipboard (type: {:?})",
            text.len(),
            yank_type
        );

        // Update the cache first
        self.cached_content = Some(YankEntry {
            text: text.clone(),
            yank_type,
        });

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

    fn paste_entry(&mut self) -> Option<YankEntry> {
        // First sync from system clipboard to get any external changes
        self.sync_from_clipboard();

        // Return cached content (now updated from clipboard)
        self.cached_content.clone()
    }

    fn paste(&mut self) -> Option<&str> {
        // First sync from system clipboard to get any external changes
        self.sync_from_clipboard();

        // Return cached content text (now updated from clipboard)
        self.cached_content
            .as_ref()
            .map(|entry| entry.text.as_str())
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
        assert_eq!(buffer.paste_entry(), None);

        // Yank some text with default type
        buffer.yank("Hello, world!".to_string()).unwrap();
        assert!(buffer.has_content());
        assert_eq!(buffer.paste(), Some("Hello, world!"));
        let entry = buffer.paste_entry().unwrap();
        assert_eq!(entry.text, "Hello, world!");
        assert_eq!(entry.yank_type, YankType::Character);

        // Yank with specific type
        buffer
            .yank_with_type("Block text".to_string(), YankType::Block)
            .unwrap();
        assert_eq!(buffer.paste(), Some("Block text"));
        let entry = buffer.paste_entry().unwrap();
        assert_eq!(entry.text, "Block text");
        assert_eq!(entry.yank_type, YankType::Block);

        // Clear buffer
        buffer.clear();
        assert!(!buffer.has_content());
        assert_eq!(buffer.paste(), None);
        assert_eq!(buffer.paste_entry(), None);
    }
}
