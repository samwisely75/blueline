//! # Editor Context
//!
//! Manages editor-specific state and operations including:
//! - Yank buffer and clipboard integration
//! - Editor settings (dcut, clipboard_enabled)
//! - Visual block insert cursor management (delegated to panes)
//!
//! This provides a clean separation of editor concerns from HTTP and UI domains.

use crate::repl::models::{ClipboardYankBuffer, MemoryYankBuffer, YankBuffer};
use anyhow::{Context, Result};

/// Editor context managing editor-specific state and operations
///
/// This context encapsulates all editor-related functionality that was previously
/// scattered throughout AppState, providing cleaner domain separation.
pub struct EditorContext {
    /// Yank buffer for copy/paste operations
    yank_buffer: Box<dyn YankBuffer>,

    /// Whether clipboard integration is enabled
    clipboard_enabled: bool,

    /// Whether d/dd/D commands should cut (yank) instead of just delete
    dcut_enabled: bool,

    /// Whether auto-format is enabled for JSON responses
    autoformat_enabled: bool,
}

impl EditorContext {
    /// Create a new EditorContext with default settings
    pub fn new() -> Self {
        Self {
            yank_buffer: Box::new(MemoryYankBuffer::new()),
            clipboard_enabled: false,
            dcut_enabled: true,        // Default to true for cut behavior
            autoformat_enabled: false, // Default to false for auto-format
        }
    }

    /// Enable or disable system clipboard integration
    pub fn set_clipboard_enabled(&mut self, enabled: bool) -> Result<()> {
        if enabled == self.clipboard_enabled {
            // No change needed
            return Ok(());
        }

        // Save any existing content before switching
        let existing_content = self.yank_buffer.paste().map(|s| s.to_string());

        // Switch yank buffer implementation
        if enabled {
            // Try to create clipboard buffer
            match ClipboardYankBuffer::new() {
                Ok(clipboard_buffer) => {
                    self.yank_buffer = Box::new(clipboard_buffer);
                    self.clipboard_enabled = true;
                    tracing::info!("Switched to system clipboard yank buffer");
                }
                Err(e) => {
                    tracing::error!("Failed to enable clipboard: {}", e);
                    return Err(e).context("Failed to access system clipboard");
                }
            }
        } else {
            // Switch back to memory buffer
            self.yank_buffer = Box::new(MemoryYankBuffer::new());
            self.clipboard_enabled = false;
            tracing::info!("Switched to memory yank buffer");
        }

        // Restore existing content if any
        if let Some(content) = existing_content {
            let _ = self.yank_buffer.yank(content);
        }

        Ok(())
    }

    /// Check if clipboard integration is enabled
    pub fn is_clipboard_enabled(&self) -> bool {
        self.clipboard_enabled
    }

    /// Enable or disable cut behavior for d/dd/D commands
    pub fn set_dcut_enabled(&mut self, enabled: bool) {
        self.dcut_enabled = enabled;
        tracing::info!("DCut mode set to: {}", if enabled { "on" } else { "off" });
    }

    /// Get whether cut behavior is enabled for d/dd/D commands
    pub fn is_dcut_enabled(&self) -> bool {
        self.dcut_enabled
    }

    /// Enable or disable auto-format for JSON responses
    pub fn set_autoformat_enabled(&mut self, enabled: bool) {
        tracing::info!("=== SETTING AUTO-FORMAT ===");
        tracing::info!(
            "Previous state: {}, New state: {}",
            self.autoformat_enabled,
            enabled
        );
        self.autoformat_enabled = enabled;
        tracing::info!(
            "Auto-format mode set to: {}",
            if enabled { "ON" } else { "OFF" }
        );
    }

    /// Get whether auto-format is enabled for JSON responses
    pub fn is_autoformat_enabled(&self) -> bool {
        self.autoformat_enabled
    }

    /// Get reference to the yank buffer
    #[allow(dead_code)]
    pub fn yank_buffer(&self) -> &dyn YankBuffer {
        self.yank_buffer.as_ref()
    }

    /// Get mutable reference to the yank buffer
    pub fn yank_buffer_mut(&mut self) -> &mut dyn YankBuffer {
        self.yank_buffer.as_mut()
    }
}

impl Default for EditorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EditorContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EditorContext")
            .field("clipboard_enabled", &self.clipboard_enabled)
            .field("dcut_enabled", &self.dcut_enabled)
            .field("autoformat_enabled", &self.autoformat_enabled)
            .field("yank_buffer", &"<dyn YankBuffer>")
            .finish()
    }
}
