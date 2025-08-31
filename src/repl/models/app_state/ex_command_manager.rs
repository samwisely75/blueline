//! # Ex Command Management
//!
//! Handles ex command buffer operations and command execution.

use super::AppState;
use crate::repl::commands::CommandEvent;
use crate::repl::view_models::PostCommandAction;
use anyhow::Result;

impl AppState {
    /// Get ex command buffer
    pub fn get_ex_command_buffer(&self) -> &str {
        self.status_line.command_buffer()
    }

    /// Add character to ex command buffer
    pub fn add_ex_command_char(&mut self, ch: char) -> Result<()> {
        self.status_line.append_to_command_buffer(ch);
        let _ = self.emit_view_event([PostCommandAction::StatusBarUpdateRequired]);
        Ok(())
    }

    /// Remove last character from ex command buffer
    pub fn backspace_ex_command(&mut self) -> Result<()> {
        self.status_line.backspace_command_buffer();
        let _ = self.emit_view_event([PostCommandAction::StatusBarUpdateRequired]);
        Ok(())
    }

    /// Clear the ex command buffer
    pub fn clear_ex_command_buffer(&mut self) {
        self.status_line.clear_command_buffer();
        let _ = self.emit_view_event([PostCommandAction::StatusBarUpdateRequired]);
    }

    /// Set the ex command buffer
    pub fn set_ex_command_buffer(&mut self, content: String) {
        self.status_line.set_command_buffer(content);
        let _ = self.emit_view_event([PostCommandAction::StatusBarUpdateRequired]);
    }

    /// Execute ex command and return resulting command events
    pub fn execute_ex_command(&mut self) -> Result<Vec<CommandEvent>> {
        let command = self.status_line.command_buffer().trim().to_string();
        let events = Vec::new();

        // Handle ex commands
        match command.as_str() {
            "" => {
                // Empty command, just exit command mode
            }
            _ => {
                // Unknown command - could emit an error event in future
                tracing::warn!("Unknown ex command: {}", command);
            }
        }

        // Clear buffer and exit command mode
        self.status_line.clear_command_buffer();
        // Restore to previous mode (Visual if we came from Visual, Normal otherwise)
        let previous_mode = self.get_previous_mode();
        self.change_mode(previous_mode)?;

        Ok(events)
    }
}
