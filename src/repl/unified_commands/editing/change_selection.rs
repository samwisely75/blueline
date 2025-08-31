//! # Change Selection Command
//!
//! Command to change selected text in Visual Block mode.
//! This command deletes the selection and enters VisualBlockInsert mode
//! for multi-cursor text replacement.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::{EditorMode, LogicalPosition};
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command to change (delete and replace) the current visual block selection
///
/// This command:
/// 1. Only works in Visual Block mode (not Visual or VisualLine)
/// 2. Deletes the selected block text
/// 3. Sets up multi-cursor positions for the deleted block range
/// 4. Transitions to VisualBlockInsert mode (unique behavior)
/// 5. Provides status feedback
pub struct ChangeSelectionCommand;

impl ChangeSelectionCommand {
    /// Create new ChangeSelectionCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for ChangeSelectionCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Only relevant for 'c' key in Visual Block mode without modifiers
        matches!(key_event.code, KeyCode::Char('c'))
            && key_event.modifiers.is_empty()
            && matches!(mode, EditorMode::VisualBlock)
            && !context.is_read_only
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        // Get the visual selection before deleting it
        let (selection_start, selection_end, _pane) = context.app_state.get_visual_selection();
        if selection_start.is_none() || selection_end.is_none() {
            tracing::warn!("No visual selection for change operation");
            context
                .app_state
                .set_status_message("No text selected".to_string());
            return Ok(vec![PostCommandAction::StatusBarUpdateRequired]);
        }

        let start = selection_start.unwrap();
        let end = selection_end.unwrap();

        // Calculate the cursor positions for Visual Block Insert mode
        // This is similar to Visual Block Insert, but we start from the deleted block
        let top_line = start.line.min(end.line);
        let bottom_line = start.line.max(end.line);
        let left_col = start.column.min(end.column);

        // Delete the selected block text first
        if let Some(deleted_text) = context.app_state.delete_selected_text()? {
            // Create cursor positions for all lines in the deleted block range
            let mut cursor_positions = Vec::new();
            for line_num in top_line..=bottom_line {
                cursor_positions.push(LogicalPosition::new(line_num, left_col));
            }

            // Set up Visual Block Insert mode with multi-cursor state
            context
                .app_state
                .set_visual_block_insert_cursors(cursor_positions.clone());

            // Switch to VisualBlockInsert mode (not regular Insert)
            context
                .app_state
                .change_mode(EditorMode::VisualBlockInsert)?;

            // Position the main cursor at the first line of the block
            context.app_state.set_cursor_position(cursor_positions[0])?;

            // Show feedback in status bar
            let char_count = deleted_text.chars().count();
            let line_count = deleted_text.lines().count();
            let message = if line_count > 1 {
                format!("Changed {line_count} lines, Visual Block Insert mode")
            } else {
                format!("Changed {char_count} characters, Visual Block Insert mode")
            };
            context.app_state.set_status_message(message);

            tracing::info!(
                "Changed {} characters ({} lines), entered Visual Block Insert mode with {} cursors",
                char_count,
                line_count,
                cursor_positions.len()
            );

            // Return view events for UI updates
            Ok(vec![
                PostCommandAction::CurrentAreaRedrawRequired,
                PostCommandAction::StatusBarUpdateRequired,
            ])
        } else {
            tracing::warn!("No text selected for changing");
            context
                .app_state
                .set_status_message("No text selected".to_string());

            Ok(vec![PostCommandAction::StatusBarUpdateRequired])
        }
    }

    fn name(&self) -> &'static str {
        "ChangeSelectionCommand"
    }
}

impl Default for ChangeSelectionCommand {
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
    fn change_selection_should_be_relevant_for_c_key_in_visual_block_mode() {
        let command = ChangeSelectionCommand::new();
        let key_event = create_key_event(KeyCode::Char('c'), KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::VisualBlock,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        assert!(command.is_relevant(key_event, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn change_selection_should_not_be_relevant_for_other_keys() {
        let command = ChangeSelectionCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::VisualBlock,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        // Test other keys
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('d'), KeyModifiers::NONE),
            EditorMode::VisualBlock,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('x'), KeyModifiers::NONE),
            EditorMode::VisualBlock,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Enter, KeyModifiers::NONE),
            EditorMode::VisualBlock,
            &context
        ));
    }

    #[test]
    fn change_selection_should_not_be_relevant_in_other_modes() {
        let command = ChangeSelectionCommand::new();
        let key_event = create_key_event(KeyCode::Char('c'), KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };

        // Test different modes
        assert!(!command.is_relevant(key_event, EditorMode::Normal, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Insert, &context));
        assert!(!command.is_relevant(key_event, EditorMode::Visual, &context));
        assert!(!command.is_relevant(key_event, EditorMode::VisualLine, &context));
        assert!(!command.is_relevant(key_event, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn change_selection_should_not_be_relevant_in_read_only_mode() {
        let command = ChangeSelectionCommand::new();
        let key_event = create_key_event(KeyCode::Char('c'), KeyModifiers::NONE);
        let context = CommandContext {
            current_mode: EditorMode::VisualBlock,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        assert!(!command.is_relevant(key_event, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn change_selection_should_not_be_relevant_with_modifiers() {
        let command = ChangeSelectionCommand::new();
        let context = CommandContext {
            current_mode: EditorMode::VisualBlock,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        // Test with various modifiers
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('c'), KeyModifiers::CONTROL),
            EditorMode::VisualBlock,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('c'), KeyModifiers::SHIFT),
            EditorMode::VisualBlock,
            &context
        ));
        assert!(!command.is_relevant(
            create_key_event(KeyCode::Char('c'), KeyModifiers::ALT),
            EditorMode::VisualBlock,
            &context
        ));
    }

    #[test]
    fn change_selection_execute_should_handle_no_selection_gracefully() {
        let command = ChangeSelectionCommand::new();
        let (mut app_state, mut services) = create_execution_context();

        // Set up VisualBlock mode but without actual selection
        app_state.change_mode(EditorMode::VisualBlock).unwrap();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        let result = command.execute(&mut context);
        assert!(result.is_ok());

        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(
            events[0],
            PostCommandAction::StatusBarUpdateRequired
        ));

        // Verify error message was set
        assert_eq!(
            context.app_state.get_status_message(),
            Some("No text selected")
        );
    }

    #[test]
    fn command_name_should_return_correct_name() {
        let command = ChangeSelectionCommand::new();
        assert_eq!(command.name(), "ChangeSelectionCommand");
    }

    #[test]
    fn default_should_create_new_instance() {
        let command = ChangeSelectionCommand::new();
        assert_eq!(command.name(), "ChangeSelectionCommand");
    }
}

// Auto-register this command using the inventory system
register_command!(ChangeSelectionCommand, "ChangeSelectionCommand");
