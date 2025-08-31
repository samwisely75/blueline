//! # Insert Character Command
//!
//! Command to insert regular characters in insert mode or visual block insert mode.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::register_command;
use crate::repl::models::pane_state::{EditorMode, Pane};
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to insert a character at cursor position in insert modes
///
/// This command handles regular character insertion in Insert and VisualBlockInsert modes.
/// It filters out control characters and keys with control modifiers.
pub struct InsertCharCommand;

impl InsertCharCommand {
    /// Create new InsertCharCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for InsertCharCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        match key_event.code {
            KeyCode::Char(ch) => {
                // Only relevant for printable characters without control modifier
                !key_event.modifiers.contains(KeyModifiers::CONTROL)
                    && !ch.is_control()
                    && matches!(mode, EditorMode::Insert | EditorMode::VisualBlockInsert)
                    && context.current_pane == Pane::Request
            }
            _ => false,
        }
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        // Only allow in Insert modes and Request pane (double-check)
        let mode = context.app_state.get_mode();
        if !matches!(mode, EditorMode::Insert | EditorMode::VisualBlockInsert) 
            || !context.app_state.is_in_request_pane() {
            return Ok(vec![]);
        }

        // In the unified command system, we return appropriate post-command actions 
        // The actual character insertion would be handled by the key processing system
        Ok(vec![
            PostCommandAction::RequestContentChanged,
            PostCommandAction::ActiveCursorUpdateRequired,
            PostCommandAction::CurrentAreaRedrawRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "InsertCharCommand"
    }
}

impl Default for InsertCharCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::AppState;
    use crate::repl::services::Services;

    #[test]
    fn insert_char_command_should_return_correct_name() {
        let command = InsertCharCommand::new();
        assert_eq!(command.name(), "InsertCharCommand");
    }

    #[test]
    fn insert_char_command_should_be_relevant_for_printable_chars_in_insert_mode() {
        let command = InsertCharCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let a_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(command.is_relevant(a_key, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_char_command_should_be_relevant_for_space_in_insert_mode() {
        let command = InsertCharCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let space_key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        assert!(command.is_relevant(space_key, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_char_command_should_be_relevant_for_capital_letters_with_shift() {
        let command = InsertCharCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let g_key = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
        assert!(command.is_relevant(g_key, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_char_command_should_be_relevant_for_japanese_characters() {
        let command = InsertCharCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test hiragana
        let hiragana_key = KeyEvent::new(KeyCode::Char('あ'), KeyModifiers::NONE);
        assert!(command.is_relevant(hiragana_key, EditorMode::Insert, &context));

        // Test katakana
        let katakana_key = KeyEvent::new(KeyCode::Char('ア'), KeyModifiers::NONE);
        assert!(command.is_relevant(katakana_key, EditorMode::Insert, &context));

        // Test kanji
        let kanji_key = KeyEvent::new(KeyCode::Char('漢'), KeyModifiers::NONE);
        assert!(command.is_relevant(kanji_key, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_char_command_should_be_relevant_in_visual_block_insert_mode() {
        let command = InsertCharCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::VisualBlockInsert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let a_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(command.is_relevant(a_key, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn insert_char_command_should_not_be_relevant_in_normal_mode() {
        let command = InsertCharCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let a_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!command.is_relevant(a_key, EditorMode::Normal, &context));
    }

    #[test]
    fn insert_char_command_should_not_be_relevant_with_control_modifier() {
        let command = InsertCharCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let ctrl_a_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(ctrl_a_key, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_char_command_should_not_be_relevant_in_response_pane() {
        let command = InsertCharCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let a_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!command.is_relevant(a_key, EditorMode::Insert, &context));
    }

    #[test]
    fn insert_char_command_should_handle_execution_in_insert_mode() {
        let command = InsertCharCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Switch to insert mode
        app_state.set_mode(EditorMode::Insert);

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);
        assert!(result.is_ok());
        
        let events = result.unwrap();
        assert!(events.len() > 0);
        assert!(events.contains(&PostCommandAction::RequestContentChanged));
        assert!(events.contains(&PostCommandAction::ActiveCursorUpdateRequired));
    }

    #[test]
    fn insert_char_command_should_not_execute_in_wrong_mode() {
        let command = InsertCharCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Keep in normal mode
        assert_eq!(app_state.get_mode(), EditorMode::Normal);

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);
        assert!(result.is_ok());
        
        let events = result.unwrap();
        assert_eq!(events.len(), 0); // Should return empty vec for wrong mode
    }

    #[test]
    fn insert_char_command_should_not_execute_in_response_pane() {
        let command = InsertCharCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Switch to insert mode and response pane
        app_state.set_mode(EditorMode::Insert);
        app_state.switch_to_other_pane(); // Switch to response pane

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);
        assert!(result.is_ok());
        
        let events = result.unwrap();
        assert_eq!(events.len(), 0); // Should return empty vec for response pane
    }

    #[test]
    fn insert_char_command_create_default_instance() {
        let command = InsertCharCommand::default();
        assert_eq!(command.name(), "InsertCharCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(InsertCharCommand, "InsertCharCommand");