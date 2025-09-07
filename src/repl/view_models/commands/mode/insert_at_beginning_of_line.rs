//! # Insert At Beginning Of Line Command
//!
//! Command to handle the 'I' (uppercase) key in Normal mode which positions
//! the cursor at the beginning of the current line and enters Insert mode,
//! enabling text insertion at the line start.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::{EditorMode, Pane};
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to handle insert at beginning of line operation
///
/// This command handles the 'I' key in Normal mode, which moves the cursor
/// to the beginning of the current line and enters Insert mode, enabling
/// text insertion at the line start.
pub struct InsertAtBeginningOfLineCommand;

impl InsertAtBeginningOfLineCommand {
    /// Create new InsertAtBeginningOfLineCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for InsertAtBeginningOfLineCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for InsertAtBeginningOfLineCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Handle 'I' key (uppercase) in Normal mode for Request pane only
        // This accepts various terminal combinations for uppercase I
        let is_uppercase_i =
            matches!(key_event.code, KeyCode::Char('I')) && key_event.modifiers.is_empty();
        let is_shift_i = matches!(key_event.code, KeyCode::Char('i'))
            && key_event.modifiers.contains(KeyModifiers::SHIFT);
        let is_uppercase_i_with_shift = matches!(key_event.code, KeyCode::Char('I'))
            && key_event.modifiers.contains(KeyModifiers::SHIFT);

        (is_uppercase_i || is_shift_i || is_uppercase_i_with_shift)
            && mode == EditorMode::Normal
            && context.current_pane == Pane::Request
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::debug!(
            "InsertAtBeginningOfLineCommand: moving cursor to line start and entering Insert mode"
        );

        // First, move cursor to the beginning of the current line
        let cursor_events = context
            .app_state
            .pane_manager
            .move_cursor_to_start_of_line();

        // Then set the mode to Insert
        context.app_state.change_mode(EditorMode::Insert)?;

        tracing::debug!(
            "InsertAtBeginningOfLineCommand: cursor moved to line start, mode changed to Insert"
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
        "InsertAtBeginningOfLineCommand"
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
    fn insert_at_beginning_of_line_command_should_return_correct_name() {
        let command = InsertAtBeginningOfLineCommand::new();
        assert_eq!(command.name(), "InsertAtBeginningOfLineCommand");
    }

    #[test]
    fn insert_at_beginning_of_line_command_should_be_relevant_for_uppercase_i() {
        let command = InsertAtBeginningOfLineCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('I'), KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn insert_at_beginning_of_line_command_should_be_relevant_for_shift_i() {
        let command = InsertAtBeginningOfLineCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::SHIFT);

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn insert_at_beginning_of_line_command_should_be_relevant_for_uppercase_i_with_shift() {
        let command = InsertAtBeginningOfLineCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('I'), KeyModifiers::SHIFT);

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn insert_at_beginning_of_line_command_should_not_be_relevant_for_lowercase_i() {
        let command = InsertAtBeginningOfLineCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn insert_at_beginning_of_line_command_should_not_be_relevant_in_insert_mode() {
        let command = InsertAtBeginningOfLineCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('I'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_at_beginning_of_line_command_should_not_be_relevant_in_response_pane() {
        let command = InsertAtBeginningOfLineCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Char('I'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn insert_at_beginning_of_line_command_should_not_be_relevant_in_visual_modes() {
        let command = InsertAtBeginningOfLineCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('I'), KeyModifiers::NONE);

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
    fn insert_at_beginning_of_line_command_should_not_be_relevant_with_other_modifiers() {
        let command = InsertAtBeginningOfLineCommand::new();
        let context = create_test_context();

        let modified_keys = vec![
            KeyEvent::new(KeyCode::Char('I'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('I'), KeyModifiers::ALT),
            KeyEvent::new(KeyCode::Char('i'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('i'), KeyModifiers::ALT),
        ];

        for key_event in modified_keys {
            assert!(
                !command.is_relevant(key_event, EditorMode::Normal, &context),
                "Should not be relevant for modified 'I' key: {key_event:?}"
            );
        }
    }

    #[test]
    fn insert_at_beginning_of_line_command_should_not_be_relevant_for_other_keys() {
        let command = InsertAtBeginningOfLineCommand::new();
        let context = create_test_context();

        let other_keys = vec![
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
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
    fn insert_at_beginning_of_line_execute_should_move_cursor_to_start_and_change_mode() {
        let command = InsertAtBeginningOfLineCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set some initial content and move cursor to middle of line
        app_state.insert_text("Hello World").unwrap();
        // Move cursor to position 5 (middle of "Hello World")
        for _ in 0..5 {
            let _ = app_state.pane_manager.move_cursor_right();
        }

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

        // Cursor should be at beginning of line (column 0)
        let cursor = context.app_state.pane_manager.get_current_cursor_position();
        assert_eq!(cursor.column, 0, "Cursor should be at beginning of line");
    }

    #[test]
    fn insert_at_beginning_of_line_command_should_work_with_empty_content() {
        let command = InsertAtBeginningOfLineCommand::new();
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

        // Cursor should remain at beginning of line
        let cursor = context.app_state.pane_manager.get_current_cursor_position();
        assert_eq!(cursor.column, 0, "Cursor should be at beginning of line");
    }

    #[test]
    fn insert_at_beginning_of_line_command_should_handle_read_only_context() {
        let command = InsertAtBeginningOfLineCommand::new();
        let read_only_context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: true, // Read-only context
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Char('I'), KeyModifiers::NONE);

        // Should still be relevant even in read-only context for mode change
        assert!(command.is_relevant(key_event, EditorMode::Normal, &read_only_context));
    }

    #[test]
    fn insert_at_beginning_of_line_command_should_work_with_selection() {
        let command = InsertAtBeginningOfLineCommand::new();
        let context_with_selection = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true, // Has active selection
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Char('I'), KeyModifiers::NONE);

        // Should still be relevant even with selection
        assert!(command.is_relevant(key_event, EditorMode::Normal, &context_with_selection));
    }

    #[test]
    fn insert_at_beginning_of_line_command_should_execute_successfully() {
        let command = InsertAtBeginningOfLineCommand::new();
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
    fn default_should_create_new_instance() {
        let command = InsertAtBeginningOfLineCommand;
        assert_eq!(command.name(), "InsertAtBeginningOfLineCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(
    InsertAtBeginningOfLineCommand,
    "InsertAtBeginningOfLineCommand"
);
