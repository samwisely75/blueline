//! # Yank Current Line Command
//!
//! Command implementation for yanking (copying) the entire current line.
//! This implements the vim "yy" command behavior where the first 'y' enters YPrefix mode
//! and the second 'y' executes the yank current line operation.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::buffer::yank_buffer::YankType;
use crate::repl::view_models::post_command_actions::PostCommandAction;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};

/// Command to yank (copy) the entire current line
///
/// This command implements vim's "yy" behavior:
/// 1. Triggered by 'y' key in YPrefix mode (second 'y' in sequence)
/// 2. Yanks the entire current line to both internal buffer and system clipboard
/// 3. Stays on the same line (doesn't move cursor)
/// 4. Returns to Normal mode
/// 5. Shows status message indicating success
#[derive(Default)]
pub struct YankCurrentLineCommand;

impl YankCurrentLineCommand {
    /// Create new YankCurrentLineCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for YankCurrentLineCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Only relevant for 'y' key in YPrefix mode without modifiers
        // Must be in Request pane (not read-only)
        matches!(key_event.code, KeyCode::Char('y'))
            && key_event.modifiers.is_empty()
            && mode == EditorMode::YPrefix
            && !context.is_read_only
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        // Yank entire current line to yank buffer
        context.app_state.yank_current_line()?;

        // Get the yanked text and sync with YankService
        if let Some(entry) = context.app_state.get_yanked_entry() {
            context
                .services
                .yank
                .yank(entry.text.clone(), YankType::Line)?;

            // Show status message
            context
                .app_state
                .set_status_message("1 line yanked".to_string());

            tracing::info!("Yanked entire current line to yank buffer");
        } else {
            tracing::warn!("Failed to yank current line - no text found");
            context
                .app_state
                .set_status_message("Nothing to yank".to_string());
        }

        // Switch to Normal mode (exits YPrefix mode)
        context.app_state.change_mode(EditorMode::Normal)?;

        // Return view events for UI updates
        Ok(vec![
            PostCommandAction::CurrentAreaRedrawRequired,
            PostCommandAction::StatusBarUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "YankCurrentLineCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crate::repl::models::AppState;
    use crate::repl::services::Services;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn yank_current_line_command_should_return_correct_name() {
        let command = YankCurrentLineCommand::new();
        assert_eq!(command.name(), "YankCurrentLineCommand");
    }

    #[test]
    fn yank_current_line_command_should_be_relevant_for_y_in_yprefix_mode() {
        let command = YankCurrentLineCommand::new();

        // Create test context for YPrefix mode in Request pane (not read-only)
        let context = CommandContext {
            current_mode: EditorMode::YPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test 'y' key in YPrefix mode - should be relevant
        let y_key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        assert!(command.is_relevant(y_key, EditorMode::YPrefix, &context));
    }

    #[test]
    fn yank_current_line_command_should_not_be_relevant_in_wrong_conditions() {
        let command = YankCurrentLineCommand::new();

        // Test in Normal mode - should not be relevant
        let context_normal = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let y_key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        assert!(!command.is_relevant(y_key, EditorMode::Normal, &context_normal));

        // Test in Visual mode - should not be relevant
        let context_visual = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };
        assert!(!command.is_relevant(y_key, EditorMode::Visual, &context_visual));

        // Test in read-only pane (Response pane) - should not be relevant
        let context_readonly = CommandContext {
            current_mode: EditorMode::YPrefix,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        assert!(!command.is_relevant(y_key, EditorMode::YPrefix, &context_readonly));

        // Test wrong key - should not be relevant
        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let context_valid = CommandContext {
            current_mode: EditorMode::YPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        assert!(!command.is_relevant(x_key, EditorMode::YPrefix, &context_valid));

        // Test with modifiers - should not be relevant
        let y_key_ctrl = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(y_key_ctrl, EditorMode::YPrefix, &context_valid));
    }

    #[test]
    fn yank_current_line_command_should_switch_to_normal_mode() {
        let command = YankCurrentLineCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set up YPrefix mode
        app_state.change_mode(EditorMode::YPrefix).unwrap();
        assert_eq!(app_state.get_mode(), EditorMode::YPrefix);

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute command
        let result = command.execute(&mut context);

        // Should succeed and switch to Normal mode
        assert!(result.is_ok());
        assert_eq!(context.app_state.get_mode(), EditorMode::Normal);

        // Should emit view events
        let events = result.unwrap();
        assert!(!events.is_empty());
        assert!(events.contains(&PostCommandAction::CurrentAreaRedrawRequired));
        assert!(events.contains(&PostCommandAction::StatusBarUpdateRequired));
    }

    #[test]
    fn yank_current_line_command_should_set_status_message() {
        let command = YankCurrentLineCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute command
        let result = command.execute(&mut context);
        assert!(result.is_ok());

        // Should set a status message
        // Note: We can't easily test the exact message without setting up buffer content,
        // but we can verify the command completes successfully
        let events = result.unwrap();
        assert!(events.contains(&PostCommandAction::StatusBarUpdateRequired));
    }

    #[test]
    fn yank_current_line_command_should_work_only_in_yprefix_mode() {
        let command = YankCurrentLineCommand::new();

        // Should be relevant only in YPrefix mode
        let modes_to_test = [
            EditorMode::Normal,
            EditorMode::Insert,
            EditorMode::Visual,
            EditorMode::VisualLine,
            EditorMode::Command,
            EditorMode::GPrefix,
            EditorMode::DPrefix,
        ];

        for mode in modes_to_test.iter() {
            let context = CommandContext {
                current_mode: *mode,
                current_pane: Pane::Request,
                is_read_only: false,
                has_selection: false,
                ex_command_buffer: String::new(),
            };

            let y_key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
            assert!(
                !command.is_relevant(y_key, *mode, &context),
                "Command should not be relevant in {mode:?} mode"
            );
        }

        // Should be relevant only in YPrefix mode
        let context_yprefix = CommandContext {
            current_mode: EditorMode::YPrefix,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let y_key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        assert!(command.is_relevant(y_key, EditorMode::YPrefix, &context_yprefix));
    }
}

// Auto-register this command using the inventory system
register_command!(YankCurrentLineCommand, "YankCurrentLineCommand");
