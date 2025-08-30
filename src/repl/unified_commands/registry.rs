//! # Unified Command Registry
//!
//! Registry that manages all Commands in the new unified architecture.
//! Replaces the old CommandRegistry by checking is_relevant() on each Command
//! and executing the first match.

use crossterm::event::KeyEvent;
use std::sync::Arc;

use crate::repl::{
    models::pane_state::EditorMode,
    unified_commands::{Command, CommandContext},
};

// Type alias for complex Command type
type CommandArc = Arc<dyn Command>;

/// Unified Command Registry that processes key events through Commands
///
/// This registry contains all available Commands and processes input by:
/// 1. Checking each Command's is_relevant() method
/// 2. Executing the first Command that matches
/// 3. Returning the ModelEvents from the Command
pub struct UnifiedCommandRegistry {
    commands: Vec<CommandArc>,
}

impl UnifiedCommandRegistry {
    /// Create a new registry with default commands
    pub fn new() -> Self {
        let mut registry = Self {
            commands: Vec::new(),
        };

        // Register default commands
        registry.register_default_commands();

        registry
    }

    /// Register all default commands
    fn register_default_commands(&mut self) {
        use crate::repl::unified_commands::{
            change_selection::ChangeSelectionCommand, cut_character::CutCharacterCommand,
            cut_current_line::CutCurrentLineCommand, cut_selection::CutSelectionCommand,
            cut_to_end_of_line::CutToEndOfLineCommand, delete_selection::DeleteSelectionCommand,
            http::HttpExecuteCommand, yank::YankSelectionCommand,
            yank_current_line::YankCurrentLineCommand,
        };

        // Add YankSelectionCommand
        self.add_command(Arc::new(YankSelectionCommand::new()));

        // Add YankCurrentLineCommand
        self.add_command(Arc::new(YankCurrentLineCommand::new()));

        // Add ChangeSelectionCommand
        self.add_command(Arc::new(ChangeSelectionCommand::new()));

        // Add DeleteSelectionCommand
        self.add_command(Arc::new(DeleteSelectionCommand::new()));

        // Add CutSelectionCommand
        self.add_command(Arc::new(CutSelectionCommand::new()));

        // Add CutCharacterCommand
        self.add_command(Arc::new(CutCharacterCommand::new()));

        // Add CutToEndOfLineCommand
        self.add_command(Arc::new(CutToEndOfLineCommand::new()));

        // Add CutCurrentLineCommand
        self.add_command(Arc::new(CutCurrentLineCommand::new()));

        // Add HttpExecuteCommand
        self.add_command(Arc::new(HttpExecuteCommand::new()));

        // TODO: Add more commands as we create them:
        // self.add_command(Arc::new(CutSelectionCommand::new()));
        // etc.
    }

    /// Add a command to the registry
    pub fn add_command(&mut self, command: CommandArc) {
        self.commands.push(command);
    }

    /// Process a key event and return ModelEvents from the first matching Command
    ///
    /// This is the main entry point that replaces the old CommandRegistry.process_event()
    pub fn process_key_event(
        &self,
        key_event: KeyEvent,
        mode: EditorMode,
        context: &CommandContext,
    ) -> Option<CommandArc> {
        // Find the first command that is relevant for this input
        for command in &self.commands {
            if command.is_relevant(key_event, mode, context) {
                tracing::debug!(
                    "Found relevant command: {} for key {:?} in mode {:?}",
                    command.name(),
                    key_event,
                    mode
                );
                return Some(Arc::clone(command));
            }
        }

        tracing::debug!(
            "No relevant command found for key {:?} in mode {:?}",
            key_event,
            mode
        );
        None
    }

    /// Get all registered commands (for testing/debugging)
    pub fn get_all_commands(&self) -> &[CommandArc] {
        &self.commands
    }

    /// Get count of registered commands
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }
}

impl Default for UnifiedCommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn unified_registry_should_create_with_default_commands() {
        let registry = UnifiedCommandRegistry::new();

        // Should have at least YankSelectionCommand and HttpExecuteCommand
        assert!(registry.command_count() >= 2);

        // Verify commands are registered
        let commands = registry.get_all_commands();
        let has_yank_command = commands
            .iter()
            .any(|cmd| cmd.name() == "YankSelectionCommand");
        let has_http_command = commands.iter().any(|cmd| cmd.name() == "HttpExecute");

        assert!(
            has_yank_command,
            "Registry should contain YankSelectionCommand"
        );
        assert!(
            has_http_command,
            "Registry should contain HttpExecuteCommand"
        );
    }

    #[test]
    fn unified_registry_should_find_relevant_command() {
        let registry = UnifiedCommandRegistry::new();

        // Test 1: Visual mode with 'y' key - should find YankSelectionCommand
        let visual_context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
        };

        let y_key = crossterm::event::KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        let result = registry.process_key_event(y_key, EditorMode::Visual, &visual_context);

        assert!(result.is_some(), "Should find YankSelectionCommand");
        let command = result.unwrap();
        assert_eq!(command.name(), "YankSelectionCommand");

        // Test 2: Normal mode with Enter key - should find HttpExecuteCommand
        let normal_context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
        };

        let enter_key = crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let result = registry.process_key_event(enter_key, EditorMode::Normal, &normal_context);

        assert!(result.is_some(), "Should find HttpExecuteCommand");
        let command = result.unwrap();
        assert_eq!(command.name(), "HttpExecute");
    }

    #[test]
    fn unified_registry_should_return_none_for_irrelevant_input() {
        let registry = UnifiedCommandRegistry::new();

        // Create context for normal mode (no visual selection)
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
        };

        // Test 'y' key in normal mode - should find no relevant command
        let y_key = crossterm::event::KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        let result = registry.process_key_event(y_key, EditorMode::Normal, &context);

        assert!(
            result.is_none(),
            "Should find no relevant command for 'y' in Normal mode"
        );
    }

    #[test]
    fn unified_registry_should_allow_adding_custom_commands() {
        let mut registry = UnifiedCommandRegistry::new();
        let initial_count = registry.command_count();

        // Add a mock command
        use anyhow::Result;

        #[derive(Default)]
        struct TestCommand;

        impl Command for TestCommand {
            fn is_relevant(
                &self,
                _key_event: KeyEvent,
                _mode: EditorMode,
                _context: &CommandContext,
            ) -> bool {
                false // Never relevant for testing
            }

            fn execute(
                &self,
                _context: &mut crate::repl::unified_commands::ExecutionContext,
            ) -> Result<Vec<crate::repl::models::events::view_events::ViewEvent>> {
                Ok(vec![])
            }

            fn name(&self) -> &'static str {
                "TestCommand"
            }
        }

        registry.add_command(Arc::new(TestCommand));

        assert_eq!(registry.command_count(), initial_count + 1);
    }
}
