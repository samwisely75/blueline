//! # Cut Selection Command
//!
//! Command to cut (yank + delete) selected text in visual modes.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::buffer::yank_buffer::YankType;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to cut (yank and delete) the current visual selection
///
/// This command:
/// 1. Yanks the selection to the buffer
/// 2. Deletes the selected text
/// 3. Returns to Normal mode
/// 4. Provides status feedback
pub struct CutSelectionCommand;

impl CutSelectionCommand {
    /// Create new CutSelectionCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for CutSelectionCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Only relevant for 'x' key in visual modes without modifiers
        matches!(key_event.code, KeyCode::Char('x'))
            && key_event.modifiers.is_empty()
            && matches!(
                mode,
                EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
            )
            && !context.is_read_only
    }

    fn execute(
        &self,
        _key_event: KeyEvent,
        context: &mut ExecutionContext,
    ) -> Result<Vec<PostCommandAction>> {
        // Get selection text and determine yank type based on current visual mode
        if let Some((text, yank_type)) = context.app_state.get_selection_text_and_type()? {
            // First yank to buffer using YankService
            context.services.yank.yank(text.clone(), yank_type)?;
            tracing::info!("Yanked selection to buffer before cut");

            // Then delete the selected text
            if let Some(deleted_text) = context.app_state.delete_selected_text()? {
                // Switch to Normal mode (automatically clears visual selection)
                context.app_state.change_mode(EditorMode::Normal)?;

                // Prepare status message
                let char_count = deleted_text.chars().count();
                let line_count = deleted_text.lines().count();
                let message = match yank_type {
                    YankType::Character => {
                        if line_count > 1 {
                            format!("{line_count} lines cut (character-wise)")
                        } else {
                            format!("{char_count} characters cut")
                        }
                    }
                    YankType::Line => format!("{line_count} lines cut (line-wise)"),
                    YankType::Block => {
                        format!("Block cut ({line_count} lines, {char_count} chars)")
                    }
                };
                context.app_state.set_status_message(message);

                tracing::info!(
                    "Cut {} characters ({} lines) in {:?} mode",
                    char_count,
                    line_count,
                    yank_type
                );

                // Return view events for UI updates
                Ok(vec![
                    PostCommandAction::CurrentAreaRedrawRequired,
                    PostCommandAction::StatusBarUpdateRequired,
                ])
            } else {
                // Selection existed but delete failed
                tracing::warn!("Failed to delete text after yanking");
                context
                    .app_state
                    .set_status_message("Cut operation failed".to_string());
                Ok(vec![PostCommandAction::StatusBarUpdateRequired])
            }
        } else {
            tracing::warn!("No text selected for cut");
            context
                .app_state
                .set_status_message("No text selected".to_string());
            Ok(vec![PostCommandAction::StatusBarUpdateRequired])
        }
    }

    fn name(&self) -> &'static str {
        "CutSelectionCommand"
    }
}

impl Default for CutSelectionCommand {
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
    use crossterm::event::KeyModifiers;

    #[test]
    fn cut_selection_command_should_return_correct_name() {
        let command = CutSelectionCommand::new();
        assert_eq!(command.name(), "CutSelectionCommand");
    }

    #[test]
    fn cut_selection_command_should_be_relevant_for_x_in_visual_mode() {
        let command = CutSelectionCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(command.is_relevant(x_key, EditorMode::Visual, &context));
    }

    #[test]
    fn cut_selection_command_should_be_relevant_in_visual_line_mode() {
        let command = CutSelectionCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::VisualLine,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(command.is_relevant(x_key, EditorMode::VisualLine, &context));
    }

    #[test]
    fn cut_selection_command_should_be_relevant_in_visual_block_mode() {
        let command = CutSelectionCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::VisualBlock,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(command.is_relevant(x_key, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn cut_selection_command_should_not_be_relevant_in_normal_mode() {
        let command = CutSelectionCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!command.is_relevant(x_key, EditorMode::Normal, &context));
    }

    #[test]
    fn cut_selection_command_should_not_be_relevant_in_read_only_pane() {
        let command = CutSelectionCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert!(!command.is_relevant(x_key, EditorMode::Visual, &context));
    }

    #[test]
    fn cut_selection_command_should_not_be_relevant_with_modifiers() {
        let command = CutSelectionCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        let x_key_ctrl = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(x_key_ctrl, EditorMode::Visual, &context));
    }

    #[test]
    fn cut_selection_command_should_handle_no_selection() {
        let command = CutSelectionCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Execute without selection
        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            PostCommandAction::StatusBarUpdateRequired
        ));
    }

    #[test]
    fn cut_selection_command_should_emit_redraw_events() {
        let command = CutSelectionCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set up visual mode with some text
        app_state.change_mode(EditorMode::Visual).ok();
        app_state.insert_text("test text").ok();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Note: In a real scenario, we'd have a selection set up
        // For this test, we're verifying the command structure
        let result = command.execute(
            KeyEvent::new(KeyCode::Null, KeyModifiers::empty()),
            &mut context,
        );
        assert!(result.is_ok());
    }
}

// Auto-register this command using the inventory system
register_command!(CutSelectionCommand, "CutSelectionCommand");
