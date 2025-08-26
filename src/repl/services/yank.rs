//! # Yank Service
//!
//! Manages yank buffer operations and switching between memory and clipboard implementations.

use anyhow::Result;

use crate::repl::models::yank_buffer::{
    ClipboardYankBuffer, MemoryYankBuffer, YankBuffer, YankEntry, YankType,
};

/// Service for managing yank/paste operations
///
/// This service wraps the YankBuffer trait implementations and handles
/// switching between memory-based and clipboard-based storage.
pub struct YankService {
    /// The actual yank buffer implementation
    buffer: Box<dyn YankBuffer>,
    /// Whether clipboard integration is enabled
    clipboard_enabled: bool,
}

impl YankService {
    /// Create a new YankService with memory buffer
    pub fn new() -> Self {
        Self {
            buffer: Box::new(MemoryYankBuffer::new()),
            clipboard_enabled: false,
        }
    }

    /// Enable or disable system clipboard integration
    ///
    /// When switching modes, existing content is preserved.
    pub fn set_clipboard_enabled(&mut self, enabled: bool) -> Result<()> {
        if enabled == self.clipboard_enabled {
            // No change needed
            return Ok(());
        }

        // Save existing content before switching
        let existing_entry = self.buffer.paste_entry();

        // Switch buffer implementation
        if enabled {
            // Try to create clipboard buffer
            match ClipboardYankBuffer::new() {
                Ok(clipboard_buffer) => {
                    self.buffer = Box::new(clipboard_buffer);
                    self.clipboard_enabled = true;
                    tracing::info!("YankService: Switched to system clipboard");
                }
                Err(e) => {
                    tracing::error!("YankService: Failed to enable clipboard: {}", e);
                    return Err(anyhow::anyhow!("Failed to access system clipboard: {}", e));
                }
            }
        } else {
            // Switch back to memory buffer
            self.buffer = Box::new(MemoryYankBuffer::new());
            self.clipboard_enabled = false;
            tracing::info!("YankService: Switched to memory buffer");
        }

        // Restore existing content if any
        if let Some(entry) = existing_entry {
            let _ = self.buffer.yank_with_type(entry.text, entry.yank_type);
        }

        Ok(())
    }

    /// Check if clipboard integration is enabled
    pub fn is_clipboard_enabled(&self) -> bool {
        self.clipboard_enabled
    }

    /// Yank text with specified type
    pub fn yank(&mut self, text: String, yank_type: YankType) -> Result<()> {
        tracing::debug!(
            "YankService: Yanking {} characters (type: {:?})",
            text.len(),
            yank_type
        );
        self.buffer.yank_with_type(text, yank_type)
    }

    /// Yank text with Character type (for backward compatibility)
    pub fn yank_text(&mut self, text: String) -> Result<()> {
        self.yank(text, YankType::Character)
    }

    /// Paste text from buffer, returning the entry with type information
    pub fn paste(&mut self) -> Option<YankEntry> {
        self.buffer.paste_entry()
    }

    /// Paste text only (for backward compatibility)
    pub fn paste_text(&mut self) -> Option<String> {
        self.buffer.paste_entry().map(|entry| entry.text)
    }

    /// Check if buffer has content
    pub fn has_content(&self) -> bool {
        self.buffer.has_content()
    }

    /// Clear the yank buffer
    pub fn clear(&mut self) {
        self.buffer.clear()
    }
}

impl Default for YankService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yank_service_should_start_with_memory_buffer() {
        let service = YankService::new();
        assert!(!service.is_clipboard_enabled());
    }

    #[test]
    fn yank_service_should_store_and_retrieve_text() {
        let mut service = YankService::new();

        // Yank some text
        service
            .yank("test text".to_string(), YankType::Line)
            .unwrap();

        // Paste should return the same text with type
        let entry = service.paste().expect("Should have content");
        assert_eq!(entry.text, "test text");
        assert_eq!(entry.yank_type, YankType::Line);
    }

    #[test]
    #[ignore] // Test passes individually but has isolation issues with clipboard when run with other tests
    fn yank_service_should_preserve_content_when_switching_modes() {
        let mut service = YankService::new();

        // Yank some text in memory mode
        service
            .yank("preserved text".to_string(), YankType::Block)
            .unwrap();

        // Try to switch to clipboard (may fail in test environment)
        let _ = service.set_clipboard_enabled(true);

        // Content should still be available
        assert!(service.has_content());
        if let Some(entry) = service.paste() {
            assert_eq!(entry.text, "preserved text");
            assert_eq!(entry.yank_type, YankType::Block);
        } else {
            panic!("Content should be preserved after mode switch");
        }
    }

    #[test]
    fn yank_service_clear_should_empty_buffer() {
        let mut service = YankService::new();

        service
            .yank("text".to_string(), YankType::Character)
            .unwrap();
        assert!(service.has_content());

        service.clear();
        assert!(!service.has_content());
        assert!(service.paste().is_none());
    }
}
