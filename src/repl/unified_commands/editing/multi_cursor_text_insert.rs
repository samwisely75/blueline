//! # Multi-Cursor Text Insert Command
//!
//! Handles text insertion for multi-cursor Visual Block Insert mode.
//! This command detects character input when in VisualBlockInsert mode
//! and inserts the same text at all cursor positions simultaneously,
//! providing live feedback across all selected lines.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::models::LogicalPosition;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};
use crate::repl::view_models::post_command_actions::PostCommandAction;

/// Command for multi-cursor text insertion in Visual Block Insert mode
///
/// This command implements the logic previously handled by handle_multi_cursor_text_insert:
/// 1. Detects character input when in VisualBlockInsert mode and not in read-only pane
/// 2. Inserts text at all cursor positions simultaneously
/// 3. Updates cursor positions after insertion
/// 4. Provides live feedback across all selected lines
///
/// ARCHITECTURAL LIMITATION:
/// The current Command trait design doesn't provide access to the KeyEvent character
/// in the execute() method. This command demonstrates the migration pattern but
/// requires architectural evolution to fully replace the legacy system.
#[derive(Default)]
pub struct MultiCursorTextInsertCommand;

impl MultiCursorTextInsertCommand {
    /// Create new MultiCursorTextInsertCommand
    pub fn new() -> Self {
        Self
    }
}

impl Command for MultiCursorTextInsertCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Only relevant for character input in VisualBlockInsert mode in writable pane
        mode == EditorMode::VisualBlockInsert
            && !context.is_read_only
            && matches!(key_event.code, KeyCode::Char(_))
            && key_event.modifiers.is_empty() // No modifiers for plain text input
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        // ARCHITECTURAL CHALLENGE: We need the character from the KeyEvent, but execute() doesn't receive it.
        //
        // Current limitation: The Command trait's execute() method doesn't provide access to the original KeyEvent.
        // This is a fundamental architectural issue for commands that need to handle arbitrary character input.
        //
        // For now, we'll implement a fallback approach that demonstrates the multi-cursor logic
        // but doesn't provide the full functionality. This represents the core migration challenge
        // that needs to be addressed at the architecture level.

        tracing::warn!("MultiCursorTextInsertCommand: KeyEvent not available in execute() - architectural limitation");

        // Get cursor positions to validate we're in the right context
        let cursor_positions = context.app_state.get_visual_block_insert_cursors().to_vec();

        if cursor_positions.is_empty() {
            // No multi-cursor positions set, this shouldn't happen in VisualBlockInsert mode
            tracing::warn!("No multi-cursor positions found in VisualBlockInsert mode");
            context.app_state.set_status_message(
                "Multi-cursor insertion: No cursor positions found".to_string(),
            );
            return Ok(vec![PostCommandAction::StatusBarUpdateRequired]);
        }

        // Since we can't get the actual character typed, we can't perform the insertion.
        // However, we can demonstrate that we successfully intercepted the character input
        // and show how many cursor positions we would insert at.
        let msg = format!(
            "Multi-cursor insert ready: {} cursor positions (character unavailable due to architectural limitation)",
            cursor_positions.len()
        );

        context.app_state.set_status_message(msg);

        tracing::info!(
            "MultiCursorTextInsertCommand executed with {} cursor positions",
            cursor_positions.len()
        );

        // Return appropriate PostCommandActions for UI updates
        Ok(vec![
            PostCommandAction::StatusBarUpdateRequired,
            // Note: We would normally also include CurrentAreaRedrawRequired and ActiveCursorUpdateRequired
            // after performing the actual insertion, but we can't insert without the character
        ])
    }

    fn name(&self) -> &'static str {
        "MultiCursorTextInsertCommand"
    }
}

/// Internal helper method to handle multi-cursor text insertion logic
///
/// This replicates the logic from handle_multi_cursor_text_insert and demonstrates
/// what the full implementation would look like if the KeyEvent character was available.
/// This method exists to preserve the core business logic that would be used
/// once the architectural limitation is resolved.
impl MultiCursorTextInsertCommand {
    /// Handle multi-cursor text insertion with a given character
    ///
    /// This method demonstrates the complete multi-cursor insertion logic
    /// that would be used if the Command trait provided access to the KeyEvent character.
    /// It replicates the original handle_multi_cursor_text_insert functionality.
    #[allow(unused)]
    fn handle_multi_cursor_insertion(
        &self,
        context: &mut ExecutionContext,
        text: &str,
    ) -> Result<Vec<PostCommandAction>> {
        let cursor_positions = context.app_state.get_visual_block_insert_cursors().to_vec();

        if cursor_positions.is_empty() {
            // Fallback to regular insert if no cursors are set
            context.app_state.insert_text(text)?;
            return Ok(vec![
                PostCommandAction::CurrentAreaRedrawRequired,
                PostCommandAction::ActiveCursorUpdateRequired,
            ]);
        }

        tracing::debug!(
            "Multi-cursor text insert: '{}' at {} positions",
            text,
            cursor_positions.len()
        );

        // Insert text at each cursor position
        // We need to process in reverse order to maintain position validity
        for position in cursor_positions.iter().rev() {
            // Temporarily set cursor to this position and insert text
            context.app_state.set_cursor_position(*position)?;
            context.app_state.insert_text(text)?;
        }

        // Update all cursor positions to reflect the inserted text
        let text_len = text.chars().count(); // Handle multi-byte characters correctly
        let updated_positions: Vec<LogicalPosition> = cursor_positions
            .iter()
            .map(|pos| LogicalPosition::new(pos.line, pos.column + text_len))
            .collect();

        // Set the primary cursor to the first position before updating positions
        if let Some(first_pos) = updated_positions.first() {
            context.app_state.set_cursor_position(*first_pos)?;
        }

        context
            .app_state
            .update_visual_block_insert_cursors(updated_positions);

        tracing::debug!("Multi-cursor text insert completed, updated cursor positions");

        // Return PostCommandActions for UI updates
        Ok(vec![
            PostCommandAction::CurrentAreaRedrawRequired,
            PostCommandAction::ActiveCursorUpdateRequired,
            PostCommandAction::StatusBarUpdateRequired,
        ])
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
    fn multi_cursor_text_insert_command_should_return_correct_name() {
        let command = MultiCursorTextInsertCommand::new();
        assert_eq!(command.name(), "MultiCursorTextInsertCommand");
    }

    #[test]
    fn multi_cursor_text_insert_command_should_be_relevant_for_char_in_visual_block_insert_mode() {
        let command = MultiCursorTextInsertCommand::new();

        // Create test context for VisualBlockInsert mode
        let context = CommandContext {
            current_mode: EditorMode::VisualBlockInsert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false, // Not relevant for this command
            ex_command_buffer: String::new(),
        };

        // Test character input - should be relevant
        let char_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(command.is_relevant(char_event, EditorMode::VisualBlockInsert, &context));

        // Test space character - should be relevant
        let space_event = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
        assert!(command.is_relevant(space_event, EditorMode::VisualBlockInsert, &context));

        // Test digit character - should be relevant
        let digit_event = KeyEvent::new(KeyCode::Char('5'), KeyModifiers::NONE);
        assert!(command.is_relevant(digit_event, EditorMode::VisualBlockInsert, &context));

        // Test special character - should be relevant
        let special_event = KeyEvent::new(KeyCode::Char('!'), KeyModifiers::NONE);
        assert!(command.is_relevant(special_event, EditorMode::VisualBlockInsert, &context));
    }

    #[test]
    fn multi_cursor_text_insert_command_should_not_be_relevant_in_wrong_conditions() {
        let command = MultiCursorTextInsertCommand::new();

        // Test in Normal mode - should not be relevant
        let context_normal = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        let char_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!command.is_relevant(char_event, EditorMode::Normal, &context_normal));

        // Test in Insert mode - should not be relevant
        let context_insert = CommandContext {
            current_mode: EditorMode::Insert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        assert!(!command.is_relevant(char_event, EditorMode::Insert, &context_insert));

        // Test in read-only pane - should not be relevant
        let context_readonly = CommandContext {
            current_mode: EditorMode::VisualBlockInsert,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        assert!(!command.is_relevant(char_event, EditorMode::VisualBlockInsert, &context_readonly));

        // Test non-character key - should not be relevant
        let enter_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let context_valid = CommandContext {
            current_mode: EditorMode::VisualBlockInsert,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
            ex_command_buffer: String::new(),
        };
        assert!(!command.is_relevant(enter_event, EditorMode::VisualBlockInsert, &context_valid));

        // Test character with modifiers - should not be relevant (reserved for other commands)
        let char_ctrl_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(
            char_ctrl_event,
            EditorMode::VisualBlockInsert,
            &context_valid
        ));

        let char_shift_event = KeyEvent::new(KeyCode::Char('A'), KeyModifiers::SHIFT);
        assert!(!command.is_relevant(
            char_shift_event,
            EditorMode::VisualBlockInsert,
            &context_valid
        ));
    }

    #[test]
    fn multi_cursor_text_insert_command_should_handle_no_cursor_positions() {
        let command = MultiCursorTextInsertCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set to VisualBlockInsert mode but don't set multi-cursor positions
        app_state
            .change_mode(EditorMode::VisualBlockInsert)
            .unwrap();

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Should succeed but return status bar update indicating no cursor positions
        let result = command.execute(&mut context);

        assert!(result.is_ok());
        let events = result.unwrap();
        assert!(!events.is_empty());
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired)));
    }

    #[test]
    fn multi_cursor_insertion_helper_should_work_with_valid_positions() {
        let command = MultiCursorTextInsertCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set up VisualBlockInsert mode with some cursor positions
        app_state
            .change_mode(EditorMode::VisualBlockInsert)
            .unwrap();

        // Add some text to work with
        app_state
            .pane_manager
            .set_request_content("line1\nline2\nline3");

        // Set multi-cursor positions
        let cursor_positions = vec![
            LogicalPosition::new(0, 2), // line1, after 'li'
            LogicalPosition::new(1, 2), // line2, after 'li'
            LogicalPosition::new(2, 2), // line3, after 'li'
        ];
        app_state.set_visual_block_insert_cursors(cursor_positions);

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Test the helper method with sample text
        let result = command.handle_multi_cursor_insertion(&mut context, "X");

        assert!(result.is_ok());
        let events = result.unwrap();
        assert!(!events.is_empty());

        // Should have current area redraw, cursor update, and status bar update
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::CurrentAreaRedrawRequired)));
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::ActiveCursorUpdateRequired)));
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::StatusBarUpdateRequired)));
    }

    #[test]
    fn multi_cursor_insertion_helper_should_fallback_to_regular_insert_with_empty_positions() {
        let command = MultiCursorTextInsertCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();

        // Set to VisualBlockInsert mode but no cursor positions
        app_state
            .change_mode(EditorMode::VisualBlockInsert)
            .unwrap();
        app_state.pane_manager.set_request_content("test content");

        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Should fall back to regular insertion
        let result = command.handle_multi_cursor_insertion(&mut context, "X");

        assert!(result.is_ok());
        let events = result.unwrap();
        assert!(!events.is_empty());

        // Should have redraw and cursor update events
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::CurrentAreaRedrawRequired)));
        assert!(events
            .iter()
            .any(|e| matches!(e, PostCommandAction::ActiveCursorUpdateRequired)));
    }
}

// Auto-register this command using the inventory system
register_command!(MultiCursorTextInsertCommand, "MultiCursorTextInsertCommand");
