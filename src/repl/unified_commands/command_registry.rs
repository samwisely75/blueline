//! Dynamic Command Registry using Inventory
//!
//! This module provides automatic command discovery and registration using the inventory crate.
//! Commands can self-register by using the `register_command!` macro, eliminating the need
//! for manual registry updates and preventing merge conflicts.

use crate::repl::unified_commands::Command;
use inventory;
use std::sync::Arc;

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

/// Macro for self-registering commands
///
/// Usage in command modules:
/// ```rust
/// register_command!(YankSelectionCommand, "YankSelectionCommand");
/// ```
#[macro_export]
macro_rules! register_command {
    ($command_type:ty, $name:literal) => {
        inventory::submit! {
            $crate::repl::unified_commands::command_registry::CommandEntry {
                name: $name,
                factory: || Box::new(<$command_type>::new()),
            }
        }
    };
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_discover_registered_commands() {
        let commands = register_all_commands();

        // Should have at least the existing commands
        assert!(
            !commands.is_empty(),
            "Should discover at least some commands"
        );

        // Check that commands have unique names
        let mut names = std::collections::HashSet::new();
        for cmd in &commands {
            assert!(
                names.insert(cmd.name()),
                "Duplicate command name: {}",
                cmd.name()
            );
        }
    }
}
