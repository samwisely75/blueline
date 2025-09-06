//! # Paste At Cursor Command
//!
//! Command to paste yanked text at current cursor position.
//! This handles 'P' key presses in Normal mode (Vim's paste before cursor).

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

// use crate::register_command; // Disabled - see note at bottom
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to paste yanked text at current cursor position
///
/// This command:
/// 1. Handles 'P' key in Normal mode without modifiers
/// 2. Uses YankService to retrieve paste content
/// 3. Uses AppState's paste_with_type() method for business logic
/// 4. Only works in writable panes (not read-only)
/// 5. Displays status messages for user feedback
pub struct PasteAtCursorCommand;

impl PasteAtCursorCommand {
    /// Create new PasteAtCursorCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for PasteAtCursorCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // 'P' (uppercase) key in Normal mode without modifiers in writable panes
        matches!(key_event.code, KeyCode::Char('P'))
            && key_event.modifiers.is_empty()
            && mode == EditorMode::Normal
            && !context.is_read_only
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Get from YankService, not the old app_state buffer!
        if let Some(yank_entry) = context.services.yank.paste() {
            tracing::debug!(
                "Retrieved yank entry with type: {:?}, text length: {}",
                yank_entry.yank_type,
                yank_entry.text.len()
            );

            // Paste the text at current position (before cursor) using type-aware paste
            if let Err(e) = context.app_state.paste_with_type(&yank_entry) {
                tracing::error!("Failed to paste at cursor: {}", e);
                context
                    .app_state
                    .set_status_message(format!("Failed to paste: {e}"));
            } else {
                tracing::info!(
                    "Successfully pasted {} characters at cursor",
                    yank_entry.text.len()
                );
                context
                    .app_state
                    .set_status_message(format!("Pasted {} characters", yank_entry.text.len()));
            }
        } else {
            // Nothing in yank buffer
            tracing::debug!("No text in yank buffer");
            context
                .app_state
                .set_status_message("Nothing to paste".to_string());
        }

        // Return UI update events
        Ok(vec![
            PostCommandAction::CurrentAreaRedrawRequired,
            PostCommandAction::StatusBarUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "PasteAtCursorCommand"
    }
}

impl Default for PasteAtCursorCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crate::repl::models::AppState;
    use crate::repl::services::Services;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_execution_context() -> (AppState, Services) {
        let app_state = AppState::new();
        let services = Services::new();
        (app_state, services)
    }

    fn create_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn paste_at_cursor_should_be_relevant_for_uppercase_p_in_normal_mode() {
        let command = PasteAtCursorCommand::new();
        let key_event = create_key_event(KeyCode::Char('P'), KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        assert!(command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn paste_at_cursor_should_not_be_relevant_for_lowercase_p() {
        let command = PasteAtCursorCommand::new();
        let key_event = create_key_event(KeyCode::Char('p'), KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // lowercase p should not be relevant for PasteAtCursorCommand
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn paste_at_cursor_should_not_be_relevant_in_non_normal_modes() {
        let command = PasteAtCursorCommand::new();
        let key_event = create_key_event(KeyCode::Char('P'), KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Should not work in Insert mode
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));

        // Should not work in Visual modes either
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
        assert!(!command.is_relevant(key_event, EditorMode::VisualLine, &context));
        assert!(!command.is_relevant(key_event, EditorMode::VisualBlock, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Command, &context));
    }

    #[test]
    fn paste_at_cursor_should_not_be_relevant_in_read_only_panes() {
        let command = PasteAtCursorCommand::new();
        let key_event = create_key_event(KeyCode::Char('P'), KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Should not work in read-only panes
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
    }

    #[test]
    fn paste_at_cursor_should_not_be_relevant_with_modifiers() {
        let command = PasteAtCursorCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test 'P' with various modifiers - should not be relevant
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('P'), KeyModifiers::CONTROL),
            EditorMode::Normal,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('P'), KeyModifiers::SHIFT),
            EditorMode::Normal,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('P'), KeyModifiers::ALT),
            EditorMode::Normal,
            &context
        ));
    }

    #[test]
    fn paste_at_cursor_should_not_be_relevant_for_other_keys() {
        let command = PasteAtCursorCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test other keys
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('y'), KeyModifiers::NONE),
            EditorMode::Normal,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('d'), KeyModifiers::NONE),
            EditorMode::Normal,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Enter, KeyModifiers::NONE),
            EditorMode::Normal,
            &context
        ));
    }

    #[test]
    fn paste_at_cursor_execute_should_handle_paste_operation() {
        let command = PasteAtCursorCommand::new();
        let (mut app_state, mut services) = create_execution_context();

        // Ensure we're in Normal mode
        app_state.change_mode(EditorMode::Normal).unwrap();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );

        // Should succeed even if nothing is in yank buffer (shows "Nothing to paste" message)
        assert!(result.is_ok());
        let events = result.unwrap();
        // Should return UI update events
        assert_eq!(events.len(), 2);
        assert!(events.contains(&PostCommandAction::CurrentAreaRedrawRequired));
        assert!(events.contains(&PostCommandAction::StatusBarUpdateRequired));
    }

    #[test]
    fn command_name_should_return_correct_name() {
        let command = PasteAtCursorCommand::new();
        assert_eq!(command.name(), "PasteAtCursorCommand");
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = PasteAtCursorCommand::new();
        assert_eq!(command.name(), "PasteAtCursorCommand");
    }
}

// NOTE: This command is superseded by PasteBeforeCommand in paste.rs
// which handles both Normal and Visual Block modes properly.
// Temporarily disabled to avoid conflicts.
// register_command!(PasteAtCursorCommand, "PasteAtCursorCommand");
