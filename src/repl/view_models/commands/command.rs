//! # Command Pattern Infrastructure
//!
//! New Command Pattern where Commands own their business logic and emit ModelEvents.
//! This is simpler than the previous approach - no services layer initially,
//! just clean Commands that work with ViewModel directly.

use anyhow::Result;
use crossterm::event::KeyEvent;

use crate::repl::{
    events::{EditorMode, Pane},
    view_models::{commands::events::ModelEvent, ViewModel},
};

/// Command trait for the new Command Pattern architecture
///
/// Commands own their business logic and emit ModelEvents describing
/// what state changes occurred. Commands have mutable access to ViewModel
/// to perform their operations.
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

    /// Execute the command with mutable access to ViewModel
    ///
    /// Commands should perform their business logic and return ModelEvents
    /// describing what state changes occurred. The events are semantic
    /// (describe WHAT happened) rather than display-specific.
    fn handle(&self, view_model: &mut ViewModel) -> Result<Vec<ModelEvent>>;

    /// Get command name for debugging and logging
    fn name(&self) -> &'static str;
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
    /// Create CommandContext from current ViewModel state
    pub fn from_view_model(view_model: &ViewModel) -> Self {
        Self {
            current_mode: view_model.get_mode(),
            current_pane: view_model.get_current_pane(),
            is_read_only: view_model.is_in_response_pane(), // Response pane is read-only
            has_selection: view_model.get_selected_text().is_some(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::view_models::ViewModel;

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

        fn handle(&self, _view_model: &mut ViewModel) -> Result<Vec<ModelEvent>> {
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
        let mut view_model = ViewModel::new();
        let result = command.handle(&mut view_model).unwrap();
        assert_eq!(result, events);
    }

    #[test]
    fn command_context_should_capture_view_model_state() {
        let view_model = ViewModel::new();
        let context = CommandContext::from_view_model(&view_model);

        // Verify context captures current state
        assert_eq!(
            context.current_mode,
            crate::repl::events::EditorMode::Normal
        );
        assert_eq!(context.current_pane, crate::repl::events::Pane::Request);
        assert!(!context.has_selection);
    }
}
