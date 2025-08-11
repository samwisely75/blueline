//! # Ex Command Management
//!
//! Handles ex command buffer operations and command execution.

use crate::repl::commands::{CommandEvent, MovementDirection};
use crate::repl::events::ViewEvent;
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;

impl ViewModel {
    /// Get ex command buffer
    pub fn get_ex_command_buffer(&self) -> &str {
        self.status_line.command_buffer()
    }

    /// Add character to ex command buffer
    pub fn add_ex_command_char(&mut self, ch: char) -> Result<()> {
        self.status_line.append_to_command_buffer(ch);
        let _ = self.emit_view_event([ViewEvent::StatusBarUpdateRequired]);
        Ok(())
    }

    /// Remove last character from ex command buffer
    pub fn backspace_ex_command(&mut self) -> Result<()> {
        self.status_line.backspace_command_buffer();
        let _ = self.emit_view_event([ViewEvent::StatusBarUpdateRequired]);
        Ok(())
    }

    /// Execute ex command and return resulting command events
    pub fn execute_ex_command(&mut self) -> Result<Vec<CommandEvent>> {
        let command = self.status_line.command_buffer().trim().to_string();
        let mut events = Vec::new();

        // Handle ex commands
        match command.as_str() {
            "q" => {
                // Quit the application
                events.push(CommandEvent::QuitRequested);
            }
            "q!" => {
                // Force quit the application
                events.push(CommandEvent::QuitRequested);
            }
            "set wrap" => {
                // Enable word wrap
                self.pane_manager.set_wrap_enabled(true);
                let visibility_events = self.pane_manager.rebuild_display_caches_and_sync();
                let mut events = vec![ViewEvent::FullRedrawRequired];
                events.extend(visibility_events);
                let _ = self.emit_view_event(events);
            }
            "set nowrap" => {
                // Disable word wrap
                self.pane_manager.set_wrap_enabled(false);
                let visibility_events = self.pane_manager.rebuild_display_caches_and_sync();
                let mut events = vec![ViewEvent::FullRedrawRequired];
                events.extend(visibility_events);
                let _ = self.emit_view_event(events);
            }
            "show profile" => {
                // Show profile information in status bar
                events.push(CommandEvent::ShowProfileRequested);
            }
            "" => {
                // Empty command, just exit command mode
            }
            _ => {
                // Check if it's a line number command (:<number>)
                if let Ok(line_number) = command.parse::<usize>() {
                    if line_number > 0 {
                        events.push(CommandEvent::CursorMoveRequested {
                            direction: MovementDirection::LineNumber(line_number),
                            amount: 1,
                        });
                    } else {
                        tracing::warn!("Invalid line number: {}", line_number);
                    }
                } else {
                    // Unknown command - could emit an error event in future
                    tracing::warn!("Unknown ex command: {}", command);
                }
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
