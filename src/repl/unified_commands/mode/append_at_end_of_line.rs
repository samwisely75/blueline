//! # Append At End Of Line Command
//!
//! Command to handle the 'A' (uppercase) key in Normal mode which positions
//! the cursor at the end of the current line and enters Insert mode,
//! enabling text insertion at the line end (append mode).

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::{EditorMode, Pane};
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to handle append at end of line operation
///
/// This command handles the 'A' key in Normal mode, which moves the cursor
/// to the end of the current line and enters Insert mode, enabling
/// text insertion at the line end (append mode).
pub struct AppendAtEndOfLineCommand;

impl AppendAtEndOfLineCommand {
    /// Create new AppendAtEndOfLineCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for AppendAtEndOfLineCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for AppendAtEndOfLineCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Handle 'A' key (uppercase) in Normal mode for Request pane only
        // This accepts various terminal combinations for uppercase A
        let is_uppercase_a =
            matches!(key_event.code, KeyCode::Char('A')) && key_event.modifiers.is_empty();
        let is_shift_a = matches!(key_event.code, KeyCode::Char('a'))
            && key_event.modifiers.contains(KeyModifiers::SHIFT);
        let is_uppercase_a_with_shift = matches!(key_event.code, KeyCode::Char('A'))
            && key_event.modifiers.contains(KeyModifiers::SHIFT);

        (is_uppercase_a || is_shift_a || is_uppercase_a_with_shift)
            && mode == EditorMode::Normal
            && context.current_pane == Pane::Request
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::debug!(
            "AppendAtEndOfLineCommand: moving cursor to line end and entering Insert mode"
        );

        // First, move cursor to the end of the current line (for append)
        let cursor_events = context
            .app_state
            .pane_manager
            .move_cursor_to_line_end_for_append();

        // Then set the mode to Insert
        context.app_state.change_mode(EditorMode::Insert)?;

        tracing::debug!(
            "AppendAtEndOfLineCommand: cursor moved to line end for append, mode changed to Insert"
        );

        // Combine cursor movement events with mode change event
        let mut events = cursor_events;
        events.extend(vec![
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
        ]);

        Ok(events)
    }

    fn name(&self) -> &'static str {
        "AppendAtEndOfLineCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crate::repl::models::AppState;
    use crate::repl::services::Services;
    use crossterm::event::KeyModifiers;

    fn create_test_context() -> CommandContext {
        CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        }
    }

    #[test]
    fn append_at_end_of_line_command_should_return_correct_name() {
        let command = AppendAtEndOfLineCommand::new();
        assert_eq!(command.name(), "AppendAtEndOfLineCommand");
    }

    #[test]
    fn append_at_end_of_line_command_should_be_relevant_for_uppercase_a() {
        let command = AppendAtEndOfLineCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn append_at_end_of_line_command_should_be_relevant_for_shift_a() {
        let command = AppendAtEndOfLineCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::SHIFT);

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn append_at_end_of_line_command_should_be_relevant_for_uppercase_a_with_shift() {
        let command = AppendAtEndOfLineCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT);

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn append_at_end_of_line_command_should_not_be_relevant_for_lowercase_a() {
        let command = AppendAtEndOfLineCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn append_at_end_of_line_command_should_not_be_relevant_in_insert_mode() {
        let command = AppendAtEndOfLineCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn append_at_end_of_line_command_should_not_be_relevant_in_response_pane() {
        let command = AppendAtEndOfLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn append_at_end_of_line_command_should_not_be_relevant_in_visual_modes() {
        let command = AppendAtEndOfLineCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE);

        let visual_modes = vec![
            EditorMode::Visual,
            EditorMode::VisualLine,
            EditorMode::VisualBlock,
        ];

        for mode in visual_modes {
            assert!(
                !command.is_relevant(key_event, mode, &context),
                "Should not be relevant in {mode:?} mode"
            );
        }
    }

    #[test]
    fn append_at_end_of_line_command_should_not_be_relevant_with_other_modifiers() {
        let command = AppendAtEndOfLineCommand::new();
        let context = create_test_context();

        let modified_keys = vec![
            KeyEvent::new(KeyCode::Char('A'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('A'), KeyModifiers::ALT),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT),
        ];

        for key_event in modified_keys {
            assert!(
                !command.is_relevant(key_event, EditorMode::Normal, &context),
                "Should not be relevant for modified 'A' key: {key_event:?}"
            );
        }
    }

    #[test]
    fn append_at_end_of_line_command_should_not_be_relevant_for_other_keys() {
        let command = AppendAtEndOfLineCommand::new();
        let context = create_test_context();

        let other_keys = vec![
            KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE),
        ];

        for key_event in other_keys {
            assert!(
                !command.is_relevant(key_event, EditorMode::Normal, &context),
                "Should not be relevant for key: {key_event:?}"
            );
        }
    }

    #[test]
    fn append_at_end_of_line_execute_should_change_mode_and_return_events() {
        let command = AppendAtEndOfLineCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set some initial content
        app_state.insert_text("Hello World").unwrap();

        // Verify initial state is Normal mode
        assert_eq!(
            app_state.pane_manager.get_current_pane_mode(),
            EditorMode::Normal
        );

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );

        assert!(result.is_ok());
        let events = result.unwrap();

        // Should return events for cursor movement and status updates
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired)));
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::ActiveCursorUpdateRequired)));

        // Should have changed to Insert mode
        assert_eq!(
            context.app_state.pane_manager.get_current_pane_mode(),
            EditorMode::Insert
        );
    }

    #[test]
    fn append_at_end_of_line_command_should_work_with_empty_content() {
        let command = AppendAtEndOfLineCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Leave content empty (cursor is already at 0,0)

        // Verify initial state is Normal mode
        assert_eq!(
            app_state.pane_manager.get_current_pane_mode(),
            EditorMode::Normal
        );

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );

        assert!(result.is_ok());
        let events = result.unwrap();

        // Should return events even with empty content
        assert!(!events.is_empty());

        // Should have changed to Insert mode
        assert_eq!(
            context.app_state.pane_manager.get_current_pane_mode(),
            EditorMode::Insert
        );

        // Cursor should be at end of line (which is 0,0 for empty content)
        let cursor = context.app_state.pane_manager.get_current_cursor_position();
        assert_eq!(cursor.column, 0, "Cursor should be at end of empty line");
    }

    #[test]
    fn append_at_end_of_line_command_should_handle_read_only_context() {
        let command = AppendAtEndOfLineCommand::new();
        let read_only_context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: true, // Read-only context
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE);

        // Should still be relevant even in read-only context for mode change
        assert!(command.is_relevant(key_event, EditorMode::Normal, &read_only_context));
    }

    #[test]
    fn append_at_end_of_line_command_should_work_with_selection() {
        let command = AppendAtEndOfLineCommand::new();
        let context_with_selection = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true, // Has active selection
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE);

        // Should still be relevant even with selection
        assert!(command.is_relevant(key_event, EditorMode::Normal, &context_with_selection));
    }

    #[test]
    fn append_at_end_of_line_command_should_execute_successfully() {
        let command = AppendAtEndOfLineCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set some content
        app_state.insert_text("Hello World").unwrap();

        // Verify initial state is Normal mode
        assert_eq!(
            app_state.pane_manager.get_current_pane_mode(),
            EditorMode::Normal
        );

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );
        assert!(result.is_ok());

        let events = result.unwrap();

        // Should return events for cursor movement and status updates
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired)));
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::ActiveCursorUpdateRequired)));

        // Should have changed to Insert mode
        assert_eq!(
            context.app_state.pane_manager.get_current_pane_mode(),
            EditorMode::Insert
        );
    }

    #[test]
    fn append_at_end_of_line_command_should_work_with_multiline_content() {
        let command = AppendAtEndOfLineCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set multiline content
        app_state.insert_text("Line 1\nLine 2\nLine 3").unwrap();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );
        assert!(result.is_ok());

        // Should have changed to Insert mode
        assert_eq!(
            context.app_state.pane_manager.get_current_pane_mode(),
            EditorMode::Insert
        );

        // Should have returned events
        let events = result.unwrap();
        assert!(!events.is_empty());
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = AppendAtEndOfLineCommand;
        assert_eq!(command.name(), "AppendAtEndOfLineCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(AppendAtEndOfLineCommand, "AppendAtEndOfLineCommand");
