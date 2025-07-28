//! # Pane Management
//!
//! Handles pane switching, mode changes, and pane-related state management.

use crate::repl::events::{EditorMode, Pane};
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;

impl ViewModel {
    /// Get current editor mode
    pub fn get_mode(&self) -> EditorMode {
        self.editor.mode()
    }

    /// Get current active pane
    pub fn get_current_pane(&self) -> Pane {
        self.editor.current_pane()
    }

    /// Switch to a different pane
    pub fn switch_pane(&mut self, pane: Pane) -> Result<()> {
        if let Some(_event) = self.editor.set_current_pane(pane) {
            // When switching panes, we need to update cursor and status bar
            self.emit_view_event(crate::repl::events::ViewEvent::StatusBarUpdateRequired);
            self.emit_view_event(crate::repl::events::ViewEvent::CursorUpdateRequired { pane });
        }
        tracing::debug!("Switched to pane: {:?}", pane);
        Ok(())
    }

    /// Change editor mode
    pub fn change_mode(&mut self, mode: EditorMode) -> Result<()> {
        let old_mode = self.editor.mode();
        let _event = self.editor.set_mode(mode);
        // TODO: self.emit_model_event(event);

        // Clear command buffer when exiting Command mode (e.g., when pressing Escape)
        if old_mode == EditorMode::Command && mode != EditorMode::Command {
            self.ex_command_buffer.clear();
            tracing::debug!("Cleared command buffer when exiting Command mode");
        }

        // Only emit events for what actually needs updating
        self.emit_view_event(crate::repl::events::ViewEvent::StatusBarUpdateRequired);
        self.emit_view_event(crate::repl::events::ViewEvent::CursorUpdateRequired {
            pane: self.editor.current_pane(),
        });

        tracing::debug!("Changed mode to: {:?}", mode);
        Ok(())
    }

    /// Get ex command buffer
    pub fn get_ex_command_buffer(&self) -> &str {
        &self.ex_command_buffer
    }

    /// Add character to ex command buffer
    pub fn add_ex_command_char(&mut self, ch: char) -> Result<()> {
        self.ex_command_buffer.push(ch);
        self.emit_view_event(crate::repl::events::ViewEvent::StatusBarUpdateRequired);
        Ok(())
    }

    /// Remove last character from ex command buffer
    pub fn backspace_ex_command(&mut self) -> Result<()> {
        self.ex_command_buffer.pop();
        self.emit_view_event(crate::repl::events::ViewEvent::StatusBarUpdateRequired);
        Ok(())
    }

    /// Execute ex command and return resulting command events
    pub fn execute_ex_command(&mut self) -> Result<Vec<crate::repl::commands::CommandEvent>> {
        let command = self.ex_command_buffer.trim();
        let mut events = Vec::new();

        // Handle ex commands
        match command {
            "q" => {
                // Quit the application
                events.push(crate::repl::commands::CommandEvent::QuitRequested);
            }
            "q!" => {
                // Force quit the application
                events.push(crate::repl::commands::CommandEvent::QuitRequested);
            }
            "set wrap" => {
                // Enable word wrap
                if let Err(e) = self.set_wrap_enabled(true) {
                    tracing::warn!("Failed to enable word wrap: {}", e);
                }
            }
            "set nowrap" => {
                // Disable word wrap
                if let Err(e) = self.set_wrap_enabled(false) {
                    tracing::warn!("Failed to disable word wrap: {}", e);
                }
            }
            "" => {
                // Empty command, just exit command mode
            }
            _ => {
                // Unknown command - could emit an error event in future
                tracing::warn!("Unknown ex command: {}", command);
            }
        }

        // Clear buffer and exit command mode
        self.ex_command_buffer.clear();
        self.change_mode(crate::repl::events::EditorMode::Normal)?;

        Ok(events)
    }

    /// Get request pane height
    pub fn request_pane_height(&self) -> u16 {
        self.request_pane_height
    }

    /// Get response pane height
    pub fn response_pane_height(&self) -> u16 {
        if self.response.status_code().is_some() {
            self.terminal_height
                .saturating_sub(self.request_pane_height)
                .saturating_sub(2) // -2 for separator and status
        } else {
            0
        }
    }
}
