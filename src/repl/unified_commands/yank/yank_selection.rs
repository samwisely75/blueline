//! # Yank Command
//!
//! Example Command implementation for yanking selected text.
//! This demonstrates the vertical slice architecture where the Command
//! owns its business logic and emits appropriate PostCommandActions.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};

use crate::register_command;
use crate::repl::models::buffer::yank_buffer::YankType;
use crate::repl::view_models::post_command_actions::PostCommandAction;
use crate::repl::models::pane_state::EditorMode;
use crate::repl::unified_commands::{Command, CommandContext, ExecutionContext};

/// Command to yank (copy) the current visual selection
///
/// This command demonstrates the new architecture:
/// 1. Checks if there's a valid selection
/// 2. Determines the appropriate yank type based on mode
/// 3. Extracts the selected text
/// 4. Stores it in the yank buffer
/// 5. Clears the selection and returns to Normal mode
/// 6. Emits semantic ModelEvents describing what happened
#[derive(Default)]
pub struct YankSelectionCommand;

impl YankSelectionCommand {
    /// Create new YankSelectionCommand
    pub fn new() -> Self {
        Self
    }

    /// Determine yank type from editor mode
    #[allow(dead_code)]
    fn determine_yank_type(mode: EditorMode) -> YankType {
        match mode {
            EditorMode::Visual => YankType::Character,
            EditorMode::VisualLine => YankType::Line,
            EditorMode::VisualBlock => YankType::Block,
            _ => YankType::Character, // Default fallback
        }
    }
}

impl Command for YankSelectionCommand {
    fn is_relevant(&self, key_event: KeyEvent, mode: EditorMode, context: &CommandContext) -> bool {
        // Only relevant for 'y' key in visual modes without modifiers
        // Note: Removed has_selection check as visual mode always has a selection
        matches!(key_event.code, KeyCode::Char('y'))
            && key_event.modifiers.is_empty()
            && matches!(
                mode,
                EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
            )
            && !context.is_read_only
    }

    fn execute(&self, context: &mut ExecutionContext) -> Result<Vec<PostCommandAction>> {
        // Get selected text from current pane
        if let Some(text) = context.app_state.get_selected_text() {
            // Determine yank type based on current visual mode
            let current_mode = context.app_state.get_mode();
            let yank_type = match current_mode {
                EditorMode::Visual => YankType::Character,
                EditorMode::VisualLine => YankType::Line,
                EditorMode::VisualBlock => YankType::Block,
                _ => YankType::Character, // Fallback for any other mode
            };

            // Store in yank buffer using YankService
            context.services.yank.yank(text.clone(), yank_type)?;

            // Switch to Normal mode (automatically clears visual selection)
            context.app_state.change_mode(EditorMode::Normal)?;

            // Prepare status message
            let char_count = text.chars().count();
            let line_count = text.lines().count();
            let message = match yank_type {
                YankType::Character => {
                    if line_count > 1 {
                        format!("{line_count} lines yanked (character-wise)")
                    } else {
                        format!("{char_count} characters yanked")
                    }
                }
                YankType::Line => format!("{line_count} lines yanked (line-wise)"),
                YankType::Block => {
                    format!("Block yanked ({line_count} lines, {char_count} chars)")
                }
            };
            context.app_state.set_status_message(message);

            tracing::info!(
                "Yanked {} characters ({} lines) to buffer as {:?}",
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
            tracing::warn!("No text selected for yanking");
            context
                .app_state
                .set_status_message("No text selected".to_string());

            Ok(vec![PostCommandAction::StatusBarUpdateRequired])
        }
    }

    fn name(&self) -> &'static str {
        "YankSelectionCommand"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yank_selection_command_should_return_correct_name() {
        let command = YankSelectionCommand::new();
        assert_eq!(command.name(), "YankSelectionCommand");
    }

    #[test]
    fn yank_selection_command_should_be_relevant_for_y_in_visual_mode() {
        use crate::repl::models::pane_state::Pane;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let command = YankSelectionCommand::new();

        // Create test context with visual selection
        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };

        // Test 'y' key in visual mode - should be relevant
        let y_key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        assert!(command.is_relevant(y_key, EditorMode::Visual, &context));

        // Test 'y' key in visual line mode - should be relevant
        assert!(command.is_relevant(y_key, EditorMode::VisualLine, &context));

        // Test 'y' key in visual block mode - should be relevant
        assert!(command.is_relevant(y_key, EditorMode::VisualBlock, &context));
    }

    #[test]
    fn yank_selection_command_should_not_be_relevant_in_wrong_conditions() {
        use crate::repl::models::pane_state::Pane;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let command = YankSelectionCommand::new();

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

        // Test in read-only pane - should not be relevant
        let context_readonly = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: true,
            ex_command_buffer: String::new(),
        };
        assert!(!command.is_relevant(y_key, EditorMode::Visual, &context_readonly));

        // Test wrong key - should not be relevant
        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let context_valid = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
            ex_command_buffer: String::new(),
        };
        assert!(!command.is_relevant(x_key, EditorMode::Visual, &context_valid));

        // Test with modifiers - should not be relevant
        let y_key_ctrl = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL);
        assert!(!command.is_relevant(y_key_ctrl, EditorMode::Visual, &context_valid));
    }

    #[test]
    fn determine_yank_type_should_map_modes_correctly() {
        assert_eq!(
            YankSelectionCommand::determine_yank_type(EditorMode::Visual),
            YankType::Character
        );
        assert_eq!(
            YankSelectionCommand::determine_yank_type(EditorMode::VisualLine),
            YankType::Line
        );
        assert_eq!(
            YankSelectionCommand::determine_yank_type(EditorMode::VisualBlock),
            YankType::Block
        );
    }

    #[test]
    fn yank_selection_command_should_fail_gracefully_in_normal_mode() {
        use crate::repl::models::AppState;
        use crate::repl::services::Services;

        let command = YankSelectionCommand::new();
        let mut app_state = AppState::new();
        let mut services = Services::new();
        let mut context = ExecutionContext {
            app_state: &mut app_state,
            services: &mut services,
        };

        // Should succeed but return status bar update for "No text selected"
        let result = command.execute(&mut context);

        assert!(result.is_ok());
        let events = result.unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], PostCommandAction::StatusBarUpdateRequired));
    }

    #[test]
    fn yank_selection_command_should_emit_events_for_empty_selection() {
        use crate::repl::models::AppState;

        let _command = YankSelectionCommand::new();
        let _app_state = AppState::new();

        // TODO: Set up visual mode and empty selection when we have the methods
        // For now, this test documents the expected behavior

        // This test will be completed when we integrate with the actual AppState methods
    }
}

// Auto-register this command using the inventory system
register_command!(YankSelectionCommand, "YankSelectionCommand");
