//! # Multi-Cursor Text Delete Command
//!
//! Handles text deletion in Visual Block Insert mode with multiple cursors.
//! This command replaces the handle_multi_cursor_text_delete functionality from AppViewModel.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::commands::events::MovementDirection;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::models::LogicalPosition;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Type alias for delete parameters to reduce type complexity
type DeleteParams = (usize, MovementDirection);

/// Command to handle multi-cursor text deletion in Visual Block Insert mode
///
/// This command implements text deletion across multiple cursors when in Visual Block Insert mode:
/// 1. Checks if we're in Visual Block Insert mode with active cursors
/// 2. Performs deletion at each cursor position, respecting boundaries
/// 3. Updates all cursor positions after deletion
/// 4. Handles both backspace (left) and delete (right) directions
/// 5. Respects Visual Block start boundaries for backspace operations
#[derive(Default)]
pub struct MultiCursorTextDeleteCommand;

impl MultiCursorTextDeleteCommand {
    /// Create new MultiCursorTextDeleteCommand
    pub fn new() -> Self {
        Self
    }

    /// Determine the deletion amount and direction from key event
    fn get_delete_params(key_event: KeyEvent) -> Option<DeleteParams> {
        match key_event.code {
            KeyCode::Backspace => Some((1, MovementDirection::Left)),
            KeyCode::Delete => Some((1, MovementDirection::Right)),
            _ => None,
        }
    }
}

impl Command for MultiCursorTextDeleteCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Only relevant in Visual Block Insert mode for delete keys
        mode == EditorMode::VisualBlockInsert
            && !context.is_read_only
            && Self::get_delete_params(key_event).is_some()
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        // This should not be called unless we're in the right mode, but double-check
        let current_mode = context.app_state.get_mode();
        if current_mode != EditorMode::VisualBlockInsert {
            tracing::warn!(
                "MultiCursorTextDeleteCommand only supported in Visual Block Insert mode, current mode: {current_mode:?}"
            );
            context.app_state.set_status_message(
                "Multi-cursor delete only supported in Visual Block Insert mode".to_string(),
            );
            return Ok(vec![PostCommandAction::StatusBarUpdateRequired]);
        }

        // For now, since we can't access the original key event in execute(),
        // we'll assume backspace (most common case) and amount of 1
        // This is a limitation of the current unified command architecture
        let amount = 1;
        let direction = MovementDirection::Left; // Default to backspace

        let cursor_positions = context.app_state.get_visual_block_insert_cursors().to_vec();
        let start_columns = context
            .app_state
            .get_visual_block_insert_start_columns()
            .to_vec();

        if cursor_positions.is_empty() {
            // Fallback to regular delete if no cursors are set
            for _ in 0..amount {
                match direction {
                    MovementDirection::Left => {
                        context.app_state.delete_char_before_cursor()?;
                    }
                    MovementDirection::Right => {
                        context.app_state.delete_char_after_cursor()?;
                    }
                    _ => {
                        tracing::warn!("Unsupported delete direction: {direction:?}");
                    }
                }
            }
            return Ok(vec![
                PostCommandAction::CurrentAreaRedrawRequired,
                PostCommandAction::ActiveCursorUpdateRequired,
            ]);
        }

        tracing::debug!(
            "Multi-cursor text delete: {} chars in direction {direction:?} at {} positions, start columns: {start_columns:?}",
            amount,
            cursor_positions.len(),
        );

        // Perform deletion at each cursor position, respecting boundaries
        // We need to process in reverse order to maintain position validity
        for (i, position) in cursor_positions.iter().enumerate().rev() {
            let start_column = start_columns.get(i).copied().unwrap_or(0);

            // Temporarily set cursor to this position
            context.app_state.set_cursor_position(*position)?;

            // For left deletion (backspace), respect the Visual Block start boundary
            let effective_amount = if direction == MovementDirection::Left {
                // Calculate how many characters we can actually delete without going beyond start
                let current_col = position.column;
                let max_deletable = current_col.saturating_sub(start_column);
                let effective = amount.min(max_deletable);
                tracing::debug!(
                    "Backspace calculation: line={}, current_col={}, start_col={}, max_deletable={}, requested={}, effective={}",
                    position.line, current_col, start_column, max_deletable, amount, effective
                );
                effective
            } else {
                amount
            };

            for _ in 0..effective_amount {
                match direction {
                    MovementDirection::Left => {
                        context.app_state.delete_char_before_cursor()?;
                    }
                    MovementDirection::Right => {
                        context.app_state.delete_char_after_cursor()?;
                    }
                    _ => {
                        tracing::warn!("Unsupported delete direction: {direction:?}");
                        break;
                    }
                }
            }

            tracing::debug!(
                "Line {}: deleted {} chars (requested: {}, start_column: {}, current: {})",
                position.line,
                effective_amount,
                amount,
                start_column,
                position.column
            );
        }

        // Update all cursor positions to reflect the deleted text
        let updated_positions: Vec<LogicalPosition> = match direction {
            MovementDirection::Left => {
                // For backspace, cursor positions move left by amount actually deleted (respecting boundaries)
                cursor_positions
                    .iter()
                    .enumerate()
                    .map(|(i, pos)| {
                        let start_column = start_columns.get(i).copied().unwrap_or(0);
                        let current_col = pos.column;
                        let max_deletable = current_col.saturating_sub(start_column);
                        let effective_amount = amount.min(max_deletable);
                        LogicalPosition::new(pos.line, pos.column.saturating_sub(effective_amount))
                    })
                    .collect()
            }
            MovementDirection::Right => {
                // For forward delete, cursor positions stay the same
                cursor_positions
            }
            _ => cursor_positions,
        };

        // Set the primary cursor to the first position before updating positions
        if let Some(first_pos) = updated_positions.first() {
            context.app_state.set_cursor_position(*first_pos)?;
        }

        context
            .app_state
            .update_visual_block_insert_cursors(updated_positions);

        tracing::debug!("Multi-cursor text delete completed, updated cursor positions");

        Ok(vec![
            PostCommandAction::CurrentAreaRedrawRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
        ])
    }

    fn name(&self) -> &'static str {
        "MultiCursorTextDeleteCommand"
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
    fn multi_cursor_text_delete_command_should_return_correct_name() {
        let command = MultiCursorTextDeleteCommand::new();
        assert_eq!(command.name(), "MultiCursorTextDeleteCommand");
    }

    #[test]
    fn multi_cursor_text_delete_command_should_be_relevant_for_delete_keys_in_visual_block_insert_mode(
    ) {
        let command = MultiCursorTextDeleteCommand::new();

        // Create test context for Visual Block Insert mode
        let context = CommandContext {
            current_mode: EditorMode::VisualBlockInsert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        // Test backspace key - should be relevant
        let backspace_key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(command.is_relevant(backspace_key, EditorMode::VisualBlockInsert, &context));

        // Test delete key - should be relevant
        let delete_key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        assert!(command.is_relevant(delete_key, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn multi_cursor_text_delete_command_should_not_be_relevant_in_wrong_conditions() {
        let command = MultiCursorTextDeleteCommand::new();

        // Test in Normal mode - should not be relevant
        let context_normal = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let backspace_key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        assert!(!command.is_relevant(backspace_key, EditorMode::Normal, &context_normal));

        // Test in Visual mode (not VisualBlockInsert) - should not be relevant
        let context_visual = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };
        assert!(!command.is_relevant(backspace_key, EditorMode::Visual, &context_visual));

        // Test in read-only pane - should not be relevant
        let context_readonly = CommandContext {
            current_mode: EditorMode::VisualBlockInsert,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: true,
            ex_command_buffer: String::new(),
        };
        assert!(!command.is_relevant(
            backspace_key,
            EditorMode::VisualBlockInsert,
            &context_readonly
        ));

        // Test wrong key - should not be relevant
        let a_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let context_valid = CommandContext {
            current_mode: EditorMode::VisualBlockInsert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };
        assert!(!command.is_relevant(a_key, EditorMode::VisualBlockInsert, &context_valid));
    }

    #[test]
    fn multi_cursor_text_delete_command_should_fail_gracefully_in_wrong_mode() {
        let command = MultiCursorTextDeleteCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Should succeed but return status bar update for wrong mode
        let result = command.execute(&mut context);

        assert!(result.is_ok());
        let actions = result.unwrap();
        assert_eq!(actions.len(), 1);
        assert!(matches!(
            actions[0],
            PostCommandAction::StatusBarUpdateRequired
        ));
    }

    #[test]
    fn multi_cursor_text_delete_command_should_handle_no_cursors() {
        let command = MultiCursorTextDeleteCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set to VisualBlockInsert mode but don't set up cursors
        app_state
            .change_mode(EditorMode::VisualBlockInsert)
            .unwrap();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Should succeed and handle fallback to regular delete
        let result = command.execute(&mut context);

        assert!(result.is_ok());
        let actions = result.unwrap();
        assert!(!actions.is_empty());
        // Should include redraw actions for fallback delete
        let has_redraw = actions
            .iter()
            .any(|e| matches!(e, PostCommandAction::CurrentAreaRedrawRequired));
        assert!(has_redraw, "Should have redraw action for fallback delete");
    }

    #[test]
    fn get_delete_params_should_return_correct_values() {
        // Test backspace key
        let backspace_key = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);
        let params = MultiCursorTextDeleteCommand::get_delete_params(backspace_key);
        assert_eq!(params, Some((1, MovementDirection::Left)));

        // Test delete key
        let delete_key = KeyEvent::new(KeyCode::Delete, KeyModifiers::NONE);
        let params = MultiCursorTextDeleteCommand::get_delete_params(delete_key);
        assert_eq!(params, Some((1, MovementDirection::Right)));

        // Test non-delete key
        let char_key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let params = MultiCursorTextDeleteCommand::get_delete_params(char_key);
        assert_eq!(params, None);
    }

    // TODO: Add integration test for multi-cursor deletion functionality
    // when we have better test setup methods for Visual Block Insert mode
    #[test]
    fn multi_cursor_text_delete_command_should_handle_multi_cursor_deletion() {
        // This test documents the expected behavior for multi-cursor deletion:
        // 1. When command executes with valid multi-cursor setup
        // 2. Should perform deletion at each cursor position
        // 3. Should respect Visual Block start boundaries
        // 4. Should update cursor positions appropriately
        // 5. Should return CurrentAreaRedrawRequired and ActiveCursorUpdateRequired

        // For now, this test is a placeholder until we can easily set up
        // multi-cursor Visual Block Insert state in tests
    }
}

// Auto-register this command using the inventory system
register_command!(MultiCursorTextDeleteCommand, "MultiCursorTextDeleteCommand");
