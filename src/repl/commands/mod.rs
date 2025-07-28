//! # Commands Module
//!
//! Event-driven command system with trait-based context access.
//! Commands analyze events and produce CommandEvents that describe what should happen.
//! The controller applies these events to maintain proper separation of concerns.

use anyhow::Result;
use crossterm::event::KeyEvent;

// Import and re-export command event types
pub mod context;
pub mod events;

pub use context::*;
pub use events::*;

/// Command trait for event-driven architecture
pub trait Command {
    /// Check if command is relevant for current state and event
    fn is_relevant(&self, context: &CommandContext, event: &KeyEvent) -> bool;

    /// Execute command and produce events describing what should happen
    /// Commands should not mutate state directly, only produce events
    fn execute(&self, event: KeyEvent, context: &CommandContext) -> Result<Vec<CommandEvent>>;

    /// Get command name for debugging
    fn name(&self) -> &'static str;
}

/// Command trait for commands that need HTTP client access
pub trait HttpCommand {
    /// Check if command is relevant for current state and event
    fn is_relevant(&self, context: &HttpCommandContext, event: &KeyEvent) -> bool;

    /// Execute command with HTTP client access
    fn execute(&self, event: KeyEvent, context: &HttpCommandContext) -> Result<Vec<CommandEvent>>;

    /// Get command name for debugging
    fn name(&self) -> &'static str;
}

// Import command modules
pub mod app;
pub mod editing;
pub mod mode;
pub mod navigation;
pub mod pane;
pub mod request;

// Re-export all commands for easy access
pub use app::AppTerminateCommand;
pub use editing::{
    DeleteCharAtCursorCommand, DeleteCharCommand, InsertCharCommand, InsertNewLineCommand,
};
pub use mode::{
    AppendAfterCursorCommand, AppendAtEndOfLineCommand, EnterCommandModeCommand,
    EnterInsertModeCommand, ExCommandModeCommand, ExitInsertModeCommand,
    InsertAtBeginningOfLineCommand,
};
pub use navigation::{
    EnterGPrefixCommand, GoToBottomCommand, GoToTopCommand, MoveCursorDownCommand,
    MoveCursorLeftCommand, MoveCursorRightCommand, MoveCursorUpCommand, ScrollHalfPageDownCommand,
    ScrollHalfPageUpCommand, ScrollLeftCommand, ScrollPageDownCommand, ScrollPageUpCommand,
    ScrollRightCommand,
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
            // App control commands (highest priority - process first)
            Box::new(AppTerminateCommand),
            // G mode commands (high priority - must be processed before regular g handling)
            Box::new(GoToTopCommand),
            Box::new(GoToBottomCommand),
            Box::new(EnterGPrefixCommand),
            // Scroll commands (higher priority than regular movement)
            Box::new(ScrollLeftCommand),
            Box::new(ScrollRightCommand),
            Box::new(ScrollPageDownCommand),
            Box::new(ScrollPageUpCommand),
            Box::new(ScrollHalfPageDownCommand),
            Box::new(ScrollHalfPageUpCommand),
            // Movement commands
            Box::new(MoveCursorLeftCommand),
            Box::new(MoveCursorRightCommand),
            Box::new(MoveCursorUpCommand),
            Box::new(MoveCursorDownCommand),
            // Mode commands
            Box::new(EnterInsertModeCommand),
            Box::new(AppendAfterCursorCommand),
            Box::new(AppendAtEndOfLineCommand),
            Box::new(InsertAtBeginningOfLineCommand),
            Box::new(ExitInsertModeCommand),
            Box::new(EnterCommandModeCommand),
            Box::new(ExCommandModeCommand),
            // Pane commands
            Box::new(SwitchPaneCommand),
            // Editing commands
            Box::new(InsertCharCommand),
            Box::new(InsertNewLineCommand),
            Box::new(DeleteCharCommand),
            Box::new(DeleteCharAtCursorCommand),
            // Request commands
            Box::new(ExecuteRequestCommand),
        ];

        Self { commands }
    }

    /// Find and execute the first relevant command for the given event
    /// Returns the events produced by the command that should be applied
    pub fn process_event(
        &self,
        event: KeyEvent,
        context: &CommandContext,
    ) -> Result<Vec<CommandEvent>> {
        tracing::debug!("Processing key event in registry: {:?}", event);
        for command in &self.commands {
            if command.is_relevant(context, &event) {
                tracing::debug!("Found relevant command: {}", command.name());
                return command.execute(event, context);
            }
        }
        tracing::debug!("No relevant command found for event: {:?}", event);
        Ok(vec![])
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

// TODO: Update tests for new event-driven API
/*
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
}
*/
