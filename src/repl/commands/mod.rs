//! # Commands Module
//!
//! Re-exports all command implementations organized by category.
//! This module maintains the same public API while organizing commands
//! into logical groups for better maintainability.

use crate::repl::view_models::ViewModel;
use anyhow::Result;
use crossterm::event::KeyEvent;

// Re-export the Command trait
pub trait Command {
    /// Check if command is relevant for current state and event
    fn is_relevant(&self, view_model: &ViewModel, event: &KeyEvent) -> bool;

    /// Execute command by delegating to ViewModel
    fn execute(&self, event: KeyEvent, view_model: &mut ViewModel) -> Result<bool>;

    /// Get command name for debugging
    fn name(&self) -> &'static str;
}

// Import command modules
pub mod editing;
pub mod mode;
pub mod movement;
pub mod pane;
pub mod request;

// Re-export all commands for easy access
pub use editing::{DeleteCharCommand, InsertCharCommand, InsertNewLineCommand};
pub use mode::{EnterCommandModeCommand, EnterInsertModeCommand, ExitInsertModeCommand};
pub use movement::{
    MoveCursorDownCommand, MoveCursorLeftCommand, MoveCursorRightCommand, MoveCursorUpCommand,
};
pub use pane::SwitchPaneCommand;
pub use request::ExecuteRequestCommand;

/// Type alias for command collection to reduce complexity
pub type CommandCollection = Vec<Box<dyn Command>>;

/// Registry for managing all available commands
pub struct CommandRegistry {
    commands: CommandCollection,
}

impl CommandRegistry {
    /// Create new command registry with all default commands
    pub fn new() -> Self {
        let commands: CommandCollection = vec![
            // Movement commands
            Box::new(MoveCursorLeftCommand),
            Box::new(MoveCursorRightCommand),
            Box::new(MoveCursorUpCommand),
            Box::new(MoveCursorDownCommand),
            // Mode commands
            Box::new(EnterInsertModeCommand),
            Box::new(ExitInsertModeCommand),
            Box::new(EnterCommandModeCommand),
            // Pane commands
            Box::new(SwitchPaneCommand),
            // Editing commands
            Box::new(InsertCharCommand),
            Box::new(InsertNewLineCommand),
            Box::new(DeleteCharCommand),
            // Request commands
            Box::new(ExecuteRequestCommand),
        ];

        Self { commands }
    }

    /// Find and execute the first relevant command for the given event
    pub fn process_event(&self, event: KeyEvent, view_model: &mut ViewModel) -> Result<bool> {
        for command in &self.commands {
            if command.is_relevant(view_model, &event) {
                return command.execute(event, view_model);
            }
        }
        Ok(false)
    }

    /// Add a custom command to the registry
    pub fn add_command(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }

    /// Get all commands (for debugging or introspection)
    pub fn get_commands(&self) -> &CommandCollection {
        &self.commands
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::events::EditorMode;
    use crossterm::event::{KeyCode, KeyModifiers};

    fn create_test_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    #[test]
    fn registry_should_create_with_all_commands() {
        let registry = CommandRegistry::new();
        assert!(!registry.commands.is_empty());
    }

    #[test]
    fn registry_should_handle_movement_command() {
        let registry = CommandRegistry::new();
        let mut vm = ViewModel::new();
        vm.change_mode(EditorMode::Insert).unwrap();
        vm.insert_text("hello world").unwrap();

        let event = create_test_key_event(KeyCode::Left);
        let handled = registry.process_event(event, &mut vm).unwrap();

        assert!(handled);
    }

    #[test]
    fn registry_should_handle_mode_change_command() {
        let registry = CommandRegistry::new();
        let mut vm = ViewModel::new();

        let event = create_test_key_event(KeyCode::Char('i'));
        let handled = registry.process_event(event, &mut vm).unwrap();

        assert!(handled);
        assert_eq!(vm.get_mode(), EditorMode::Insert);
    }

    #[test]
    fn registry_should_return_false_for_unhandled_events() {
        let registry = CommandRegistry::new();
        let mut vm = ViewModel::new();

        let event = create_test_key_event(KeyCode::Char('z')); // No command for 'z'
        let handled = registry.process_event(event, &mut vm).unwrap();

        assert!(!handled);
    }
}
