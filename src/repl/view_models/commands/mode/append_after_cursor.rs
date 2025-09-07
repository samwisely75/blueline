//! # Append After Cursor Command
//!
//! Command to handle the 'a' key in Normal mode which positions the cursor
//! one character to the right and enters Insert mode, enabling text insertion
//! after the current cursor position.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::{EditorMode, Pane};
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to handle append after cursor operation
///
/// This command handles the 'a' key in Normal mode, which moves the cursor
/// one position to the right and enters Insert mode, enabling text insertion
/// after the current cursor position.
pub struct AppendAfterCursorCommand;

impl AppendAfterCursorCommand {
    /// Create new AppendAfterCursorCommand
    pub fn new() -> Self {
        Self
    }
}

impl Default for AppendAfterCursorCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl Command for AppendAfterCursorCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Handle 'a' key in Normal mode for Request pane only
        matches!(key_event.code, KeyCode::Char('a'))
            && mode == EditorMode::Normal
            && context.current_pane == Pane::Request
            && key_event.modifiers.is_empty()
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        tracing::debug!("AppendAfterCursorCommand: moving cursor right and entering Insert mode");

        // First, move cursor one position to the right
        context.app_state.pane_manager.move_cursor_right();

        // Then set the mode to Insert
        context.app_state.change_mode(EditorMode::Insert)?;

        tracing::debug!("AppendAfterCursorCommand: cursor moved right, mode changed to Insert");

        // Generate view update events based on what this command did
        Ok(vec![
            PostCommandAction::StatusBarUpdateRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
            PostCommandAction::PositionIndicatorUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "AppendAfterCursorCommand"
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
    fn append_after_cursor_command_should_return_correct_name() {
        let command = AppendAfterCursorCommand::new();
        assert_eq!(command.name(), "AppendAfterCursorCommand");
    }

    #[test]
    fn append_after_cursor_command_should_be_relevant_for_a_in_normal_mode() {
        let command = AppendAfterCursorCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn append_after_cursor_command_should_not_be_relevant_in_insert_mode() {
        let command = AppendAfterCursorCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
    }

    #[test]
    fn append_after_cursor_command_should_not_be_relevant_in_response_pane() {
        let command = AppendAfterCursorCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn append_after_cursor_command_should_not_be_relevant_in_visual_modes() {
        let command = AppendAfterCursorCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

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
    fn append_after_cursor_command_should_not_be_relevant_for_uppercase_a() {
        let command = AppendAfterCursorCommand::new();
        let context = create_test_context();
        let key_event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::NONE);

        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn append_after_cursor_command_should_not_be_relevant_with_modifiers() {
        let command = AppendAfterCursorCommand::new();
        let context = create_test_context();

        let modified_keys = vec![
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::SHIFT),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL),
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT),
            KeyEvent::new(
                KeyCode::Char('a'),
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
            ),
        ];

        for key_event in modified_keys {
            assert!(
                !command.is_relevant(key_event, EditorMode::Normal, &context),
                "Should not be relevant for modified 'a' key: {key_event:?}"
            );
        }
    }

    #[test]
    fn append_after_cursor_command_should_not_be_relevant_for_other_keys() {
        let command = AppendAfterCursorCommand::new();
        let context = create_test_context();

        let other_keys = vec![
            KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE),
            KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE),
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
    fn append_after_cursor_execute_should_move_cursor_and_change_mode() {
        let command = AppendAfterCursorCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set some initial content so cursor can move right
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
    fn append_after_cursor_command_should_work_with_empty_content() {
        let command = AppendAfterCursorCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Leave content empty (cursor is at 0,0)

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
    }

    #[test]
    fn append_after_cursor_command_should_handle_read_only_context() {
        let command = AppendAfterCursorCommand::new();
        let read_only_context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: true, // Read-only context
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        // Should still be relevant even in read-only context for mode change
        assert!(command.is_relevant(key_event, EditorMode::Normal, &read_only_context));
    }

    #[test]
    fn append_after_cursor_command_should_work_with_selection() {
        let command = AppendAfterCursorCommand::new();
        let context_with_selection = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true, // Has active selection
            ex_command_buffer: String::new(),
        };
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        // Should still be relevant even with selection
        assert!(command.is_relevant(key_event, EditorMode::Normal, &context_with_selection));
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = AppendAfterCursorCommand;
        assert_eq!(command.name(), "AppendAfterCursorCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(AppendAfterCursorCommand, "AppendAfterCursorCommand");
