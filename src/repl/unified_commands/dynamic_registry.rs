//! # Dynamic Command Registry
//!
//! Registry that automatically discovers and manages all Commands using the inventory system.
//! This completely replaces the old UnifiedCommandRegistry with zero-conflict dynamic discovery.

use crossterm::event::KeyEvent;
use inventory;
use std::sync::Arc;

use crate::repl::{
    models::pane_state::EditorMode,
    unified_commands::{Command, CommandContext},
};

// Type alias for complex Command type
type CommandArc = Arc<dyn Command>;

/// Command factory function type
pub type CommandFactory = fn() -> Box<dyn Command>;

/// Command registry entry for automatic discovery
#[derive(Debug)]
pub struct CommandEntry {
    pub name: &'static str,
    pub factory: CommandFactory,
}

// Global collection of all registered commands
inventory::collect!(CommandEntry);

/// Auto-register all discovered commands
#[allow(clippy::type_complexity)]
pub fn register_all_commands() -> Vec<Arc<dyn Command>> {
    let mut commands = Vec::new();

    for entry in inventory::iter::<CommandEntry> {
        tracing::debug!("Auto-registering command: {}", entry.name);
        commands.push(Arc::from((entry.factory)()));
    }

    tracing::info!("Auto-registered {} commands", commands.len());
    commands
}

/// Dynamic Command Registry that auto-discovers commands using inventory
///
/// This registry uses compile-time inventory collection to automatically discover
/// all commands that have registered themselves with `register_command!` macro.
///
/// Zero manual registration needed - commands self-register!
pub struct DynamicCommandRegistry {
    commands: Vec<CommandArc>,
}

impl DynamicCommandRegistry {
    /// Create a new registry with auto-discovered commands
    pub fn new() -> Self {
        // Auto-discover all commands using inventory
        let discovered_commands = register_all_commands();

        tracing::info!(
            "Auto-discovered {} commands via dynamic registry",
            discovered_commands.len()
        );

        Self {
            commands: discovered_commands,
        }
    }

    /// Add a command to the registry (for testing or runtime additions)
    pub fn add_command(&mut self, command: CommandArc) {
        self.commands.push(command);
    }

    /// Process a key event and return the first matching Command
    ///
    /// This is the main entry point for command processing
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

impl Default for DynamicCommandRegistry {
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
    fn dynamic_registry_should_auto_discover_commands() {
        let registry = DynamicCommandRegistry::new();

        // Should have discovered commands automatically
        assert!(
            registry.command_count() >= 13,
            "Should discover at least 13 commands, found: {}",
            registry.command_count()
        );

        // Verify some key commands are discovered
        let commands = registry.get_all_commands();
        let command_names: Vec<_> = commands.iter().map(|cmd| cmd.name()).collect();

        // Debug: Print all discovered commands
        println!("Discovered {} commands:", command_names.len());
        for name in &command_names {
            println!("  - {name}");
        }

        assert!(
            command_names.contains(&"YankSelectionCommand"),
            "Should discover YankSelectionCommand"
        );
        assert!(
            command_names.contains(&"HttpExecute"),
            "Should discover HttpExecuteCommand"
        );
        assert!(
            command_names.contains(&"EnterVisualBlockChangeModeCommand"),
            "Should discover EnterVisualBlockChangeModeCommand"
        );
        assert!(
            command_names.contains(&"MoveLeftCommand"),
            "Should discover MoveLeftCommand"
        );
        assert!(
            command_names.contains(&"ExQuitCommand"),
            "Should discover ExQuitCommand"
        );
    }

    #[test]
    fn dynamic_registry_should_find_relevant_command() {
        let registry = DynamicCommandRegistry::new();

        // Test Visual mode with 'y' key - should find YankSelectionCommand
        let visual_context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let y_key = crossterm::event::KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        let result = registry.process_key_event(y_key, EditorMode::Visual, &visual_context);

        assert!(result.is_some(), "Should find YankSelectionCommand");
        let command = result.unwrap();
        assert_eq!(command.name(), "YankSelectionCommand");
    }

    #[test]
    fn dynamic_registry_should_find_move_left_command() {
        let registry = DynamicCommandRegistry::new();

        // Test Normal mode with 'h' key - should find MoveLeftCommand
        let normal_context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let h_key = crossterm::event::KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE);
        let result = registry.process_key_event(h_key, EditorMode::Normal, &normal_context);

        assert!(result.is_some(), "Should find MoveLeftCommand for 'h' key");
        let command = result.unwrap();
        assert_eq!(command.name(), "MoveLeftCommand");

        // Test Left arrow key - should also find MoveLeftCommand
        let left_key = crossterm::event::KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
        let result = registry.process_key_event(left_key, EditorMode::Normal, &normal_context);

        assert!(
            result.is_some(),
            "Should find MoveLeftCommand for Left arrow key"
        );
        let command = result.unwrap();
        assert_eq!(command.name(), "MoveLeftCommand");
    }

    #[test]
    fn dynamic_registry_should_return_none_for_irrelevant_input() {
        let registry = DynamicCommandRegistry::new();

        // Create context for normal mode (no visual selection)
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test some key that no command should handle
        let irrelevant_key = crossterm::event::KeyEvent::new(KeyCode::F(99), KeyModifiers::NONE);
        let result = registry.process_key_event(irrelevant_key, EditorMode::Normal, &context);

        assert!(
            result.is_none(),
            "Should find no relevant command for F99 key"
        );
    }

    #[test]
    fn dynamic_registry_should_find_ex_quit_command() {
        let registry = DynamicCommandRegistry::new();

        // Test Command mode with ':q' command - should find ExQuitCommand
        let context = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: "q".to_string(),
        };

        let enter_key = crossterm::event::KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let result = registry.process_key_event(enter_key, EditorMode::Command, &context);

        assert!(
            result.is_some(),
            "Should find ExQuitCommand for ':q' + Enter"
        );
        let command = result.unwrap();
        assert_eq!(command.name(), "ExQuitCommand");

        // Test with ':q!' as well
        let context_force = CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: "q!".to_string(),
        };

        let result_force =
            registry.process_key_event(enter_key, EditorMode::Command, &context_force);
        assert!(
            result_force.is_some(),
            "Should find ExQuitCommand for ':q!' + Enter"
        );
        let command_force = result_force.unwrap();
        assert_eq!(command_force.name(), "ExQuitCommand");
    }

    #[test]
    fn dynamic_registry_should_allow_adding_custom_commands() {
        let mut registry = DynamicCommandRegistry::new();
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
                _key_event: crossterm::event::KeyEvent,
                _context: &mut crate::repl::unified_commands::ExecutionContext,
            ) -> Result<Vec<crate::repl::view_models::post_command_actions::PostCommandAction>>
            {
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
