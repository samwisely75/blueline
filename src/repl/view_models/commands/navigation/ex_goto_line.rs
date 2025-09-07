//! # Ex Goto Line Command
//!
//! Handles `:g` and `:<number>` ex commands for navigating to specific lines.
//! Supports both `:g N` (goto line N) and `:g` (goto last line) patterns.

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::view_models::commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

/// Command for handling goto line ex commands (:g, :g N, :<number>)
pub struct ExGotoLineCommand;

impl ExGotoLineCommand {
    /// Create a new ExGotoLineCommand instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExGotoLineCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl ExGotoLineCommand {
    /// Parse the command buffer to extract line number, if any
    /// Returns Some(line_number) or None for "goto last line"
    fn parse_goto_command(buffer: &str) -> Option<Option<usize>> {
        let trimmed = buffer.trim();

        // Handle direct line number commands (:<number>)
        if let Ok(line_number) = trimmed.parse::<usize>() {
            if line_number > 0 {
                return Some(Some(line_number));
            }
        }

        // Handle :g commands
        if trimmed == "g" {
            // Check if the original was exactly "g " (g followed by single space) - this is invalid
            if buffer == "g " {
                return None;
            }
            // :g without arguments means goto last line
            return Some(None);
        } else if let Some(args) = trimmed.strip_prefix("g ") {
            let arg_trimmed = args.trim();
            // Reject empty arguments (e.g., "g " with only whitespace after g)
            if arg_trimmed.is_empty() {
                return None;
            }
            // :g N means goto line N
            if let Ok(line_number) = arg_trimmed.parse::<usize>() {
                if line_number > 0 {
                    return Some(Some(line_number));
                }
            }
        }

        None
    }
}

impl Command for ExGotoLineCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Check if Enter key is pressed in Command mode
        if key_event.code != KeyCode::Enter || mode != EditorMode::Command {
            return false;
        }

        // Check if ex command buffer contains goto commands
        Self::parse_goto_command(&context.ex_command_buffer).is_some()
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        let buffer = context.app_state.get_ex_command_buffer();

        if let Some(line_number_opt) = Self::parse_goto_command(buffer) {
            // Clear the command buffer and exit command mode
            context.app_state.clear_ex_command_buffer();
            let previous_mode = context.app_state.get_previous_mode();
            context.app_state.change_mode(previous_mode)?;

            // Execute the cursor movement using PaneManager methods
            match line_number_opt {
                Some(line_number) => {
                    // Goto specific line number
                    context
                        .app_state
                        .pane_manager
                        .move_cursor_to_line(line_number);
                }
                None => {
                    // Goto last line - use move_cursor_to_document_end
                    context.app_state.pane_manager.move_cursor_to_document_end();
                }
            }

            Ok(vec![
                PostCommandAction::ActiveCursorUpdateRequired,
                PostCommandAction::PositionIndicatorUpdateRequired,
                PostCommandAction::CurrentAreaRedrawRequired,
            ])
        } else {
            // This shouldn't happen given our is_relevant check, but handle gracefully
            tracing::warn!(
                "ExGotoLineCommand executed with unexpected buffer: {}",
                buffer
            );
            Ok(vec![])
        }
    }

    fn name(&self) -> &'static str {
        "ExGotoLineCommand"
    }
}

// Register the command using modern macro
register_command!(ExGotoLineCommand, "ExGotoLineCommand");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::models::pane_state::Pane;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn create_command_context(buffer: &str) -> CommandContext {
        CommandContext {
            current_mode: EditorMode::Command,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: buffer.to_string(),
        }
    }

    fn create_key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn parse_goto_command_should_handle_direct_line_numbers() {
        assert_eq!(ExGotoLineCommand::parse_goto_command("42"), Some(Some(42)));
        assert_eq!(ExGotoLineCommand::parse_goto_command("1"), Some(Some(1)));
        assert_eq!(
            ExGotoLineCommand::parse_goto_command("100"),
            Some(Some(100))
        );

        // Invalid line numbers
        assert_eq!(ExGotoLineCommand::parse_goto_command("0"), None);
        assert_eq!(ExGotoLineCommand::parse_goto_command("abc"), None);
        assert_eq!(ExGotoLineCommand::parse_goto_command(""), None);
    }

    #[test]
    fn parse_goto_command_should_handle_g_commands() {
        // :g without arguments (goto last line)
        assert_eq!(ExGotoLineCommand::parse_goto_command("g"), Some(None));

        // :g with line number
        assert_eq!(
            ExGotoLineCommand::parse_goto_command("g 42"),
            Some(Some(42))
        );
        assert_eq!(ExGotoLineCommand::parse_goto_command("g 1"), Some(Some(1)));
        assert_eq!(
            ExGotoLineCommand::parse_goto_command("g  100  "),
            Some(Some(100))
        );

        // Invalid :g commands
        assert_eq!(ExGotoLineCommand::parse_goto_command("g 0"), None);
        assert_eq!(ExGotoLineCommand::parse_goto_command("g abc"), None);
        assert_eq!(ExGotoLineCommand::parse_goto_command("g "), None);
    }

    #[test]
    fn parse_goto_command_should_handle_whitespace() {
        assert_eq!(
            ExGotoLineCommand::parse_goto_command("  42  "),
            Some(Some(42))
        );
        assert_eq!(ExGotoLineCommand::parse_goto_command("  g  "), Some(None));
        assert_eq!(
            ExGotoLineCommand::parse_goto_command("  g  42  "),
            Some(Some(42))
        );
    }

    #[test]
    fn is_relevant_should_detect_enter_in_command_mode_with_goto_commands() {
        let command = ExGotoLineCommand;
        let enter_key = create_key_event(KeyCode::Enter);

        // Test direct line numbers
        let context = create_command_context("42");
        assert!(command.is_relevant(enter_key, EditorMode::Command, &context));

        // Test :g commands
        let context = create_command_context("g");
        assert!(command.is_relevant(enter_key, EditorMode::Command, &context));

        let context = create_command_context("g 100");
        assert!(command.is_relevant(enter_key, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_reject_non_goto_commands() {
        let command = ExGotoLineCommand;
        let enter_key = create_key_event(KeyCode::Enter);

        // Test invalid commands
        let context = create_command_context("q");
        assert!(!command.is_relevant(enter_key, EditorMode::Command, &context));

        let context = create_command_context("abc");
        assert!(!command.is_relevant(enter_key, EditorMode::Command, &context));

        let context = create_command_context("");
        assert!(!command.is_relevant(enter_key, EditorMode::Command, &context));

        let context = create_command_context("0");
        assert!(!command.is_relevant(enter_key, EditorMode::Command, &context));
    }

    #[test]
    fn is_relevant_should_require_command_mode_and_enter_key() {
        let command = ExGotoLineCommand;
        let context = create_command_context("42");

        // Should require Command mode
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Enter),
            EditorMode::Normal,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Enter),
            EditorMode::Insert,
            &context
        ));

        // Should require Enter key
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('g')),
            EditorMode::Command,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('4')),
            EditorMode::Command,
            &context
        ));
    }

    #[test]
    fn command_name_should_return_correct_name() {
        let command = ExGotoLineCommand;
        assert_eq!(command.name(), "ExGotoLineCommand");
    }
}
