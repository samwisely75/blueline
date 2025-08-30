//! # Command Pattern Infrastructure
//!
//! Command Pattern where Commands use Services for business logic and emit ModelEvents.
//! Commands receive both AppState and Services through an ExecutionContext.

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::repl::{
    models::pane_state::{EditorMode, Pane},
    models::AppState,
    services::Services,
    unified_commands::events::ModelEvent,
};

/// Command trait for the new Command Pattern architecture
///
/// Commands use Services for business logic and emit ModelEvents describing
/// what state changes occurred. Commands receive both AppState and Services
/// through an ExecutionContext.
pub trait Command: Send + Sync {
    /// Check if this command should handle the given key event
    ///
    /// This method determines command relevance based on:
    /// - Key event (key code, modifiers)
    /// - Current editor mode
    /// - Current application context
    ///
    /// Only one command should return true for any given input.
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool;

    /// Execute the command with access to AppState and Services
    ///
    /// Commands should use Services for business logic and return ModelEvents
    /// describing what state changes occurred. The events are semantic
    /// (describe WHAT happened) rather than display-specific.
    fn handle(&self, context: &mut ExecutionContext) -> Result<Vec<ModelEvent>>;

    /// Get command name for debugging and logging
    fn name(&self) -> &'static str;
}

/// Execution context for Commands containing AppState and Services
///
/// This provides Commands with access to both state (AppState) and
/// business logic services for performing operations.
pub struct ExecutionContext<'a> {
    /// Mutable access to the AppState for state management
    pub app_state: &'a mut AppState,
    /// Mutable access to Services for business operations
    pub services: &'a mut Services,
}

/// Context for Commands containing current application state
///
/// This provides Commands with read-only access to application state
/// needed for is_relevant() checks, without giving mutable access.
#[derive(Debug, Clone)]
pub struct CommandContext {
    /// Current editor mode
    pub current_mode: EditorMode,
    /// Currently active pane
    pub current_pane: Pane,
    /// Whether we're currently in a read-only pane
    pub is_read_only: bool,
    /// Whether there's an active visual selection
    pub has_selection: bool,
}

impl CommandContext {
    /// Create CommandContext from current AppState state
    pub fn from_app_state(app_state: &AppState) -> Self {
        Self {
            current_mode: app_state.get_mode(),
            current_pane: app_state.get_current_pane(),
            is_read_only: app_state.is_in_response_pane(), // Response pane is read-only
            has_selection: app_state.get_selected_text().is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::AppState;

    /// Mock command for testing the Command trait
    struct MockCommand {
        name: &'static str,
        events_to_return: Vec<ModelEvent>,
    }

    impl MockCommand {
        fn new(name: &'static str, events: Vec<ModelEvent>) -> Self {
            Self {
                name,
                events_to_return: events,
            }
        }
    }

    impl Command for MockCommand {
        fn is_relevant(
            &self,
            _key_event: KeyEvent,
            _mode: EditorMode,
            _context: &CommandContext,
        ) -> bool {
            // Mock command is always relevant for testing
            true
        }

        fn handle(&self, _context: &mut ExecutionContext) -> Result<Vec<ModelEvent>> {
            Ok(self.events_to_return.clone())
        }

        fn name(&self) -> &'static str {
            self.name
        }
    }

    #[test]
    fn command_trait_should_return_name_and_handle_events() {
        let events = vec![ModelEvent::StatusMessageSet {
            message: "Test message".to_string(),
        }];
        let command = MockCommand::new("TestCommand", events.clone());

        assert_eq!(command.name(), "TestCommand");

        // Create minimal context for testing
        let mut app_state = AppState::new();
        let mut services = crate::repl::services::Services::new();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };
        let result = command.handle(&mut context).unwrap();
        assert_eq!(result, events);
    }

    #[test]
    fn command_context_should_capture_app_state() {
        let app_state = AppState::new();
        let context = CommandContext::from_app_state(&app_state);

        // Verify context captures current state
        assert_eq!(
            context.current_mode,
            crate::repl::models::pane_state::EditorMode::Normal
        );
        assert_eq!(
            context.current_pane,
            crate::repl::models::pane_state::Pane::Request
        );
        assert!(!context.has_selection);
    }
}
