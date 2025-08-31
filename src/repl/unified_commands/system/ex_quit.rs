//! # Ex Quit Command
//!
//! Handles `:q` and `:q!` ex commands for quitting the application.
//! This demonstrates the unified command system approach for ex commands.

use crate::repl::models::events::view_events::ViewEvent;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Command for handling quit ex commands (:q, :q!)
pub struct ExQuitCommand;

impl Command for ExQuitCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Check if Enter key is pressed in Command mode
        if key_event.code != KeyCode::Enter || mode != EditorMode::Command {
            return false;
        }

        // Check if ex command buffer contains quit commands
        let buffer = context.ex_command_buffer.trim();
        matches!(buffer, "q" | "q!")
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<ViewEvent>> {
        let command = context.app_state.get_ex_command_buffer().trim();

        match command {
            "q" | "q!" => {
                // Clear the command buffer and exit command mode
                context.app_state.clear_ex_command_buffer();
                let previous_mode = context.app_state.get_previous_mode();
                context.app_state.change_mode(previous_mode)?;

                // Request application quit
                Ok(vec![ViewEvent::QuitRequested])
            }
            _ => {
                // This shouldn't happen given our is_relevant check, but handle gracefully
                tracing::warn!("ExQuitCommand executed with unexpected buffer: {}", command);
                Ok(vec![])
            }
        }
    }

    fn name(&self) -> &'static str {
        "ExQuitCommand"
    }
}

// Register the command
inventory::submit!(
    crate::repl::unified_commands::dynamic_registry::CommandEntry {
        name: "ExQuitCommand",
        factory: || Box::new(ExQuitCommand),
    }
);
