//! # Yank Command
//!
//! Example Command implementation for yanking selected text.
//! This demonstrates the vertical slice architecture where the Command
//! owns its business logic and emits appropriate ModelEvents.

use anyhow::{bail, Result};
use crossterm::event::{KeyCode, KeyEvent};

use crate::repl::events::EditorMode;
use crate::repl::view_models::commands::{
    events::{ModelEvent, YankType},
    Command, CommandContext, ExecutionContext,
};

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
        matches!(key_event.code, KeyCode::Char('y'))
            && key_event.modifiers.is_empty()
            && matches!(
                mode,
                EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
            )
            && context.has_selection
            && !context.is_read_only
    }

    fn handle(&self, context: &mut ExecutionContext) -> Result<Vec<ModelEvent>> {
        let current_pane = context.view_model.get_current_pane();
        let current_mode = context.view_model.get_mode();

        // Check if we're in a visual mode
        if !matches!(
            current_mode,
            EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
        ) {
            bail!("Yank selection only works in visual modes");
        }

        // Get selected text directly from ViewModel
        let selected_text = match context.view_model.get_selected_text() {
            Some(text) => text,
            None => {
                return Ok(vec![ModelEvent::StatusMessageSet {
                    message: "No text selected".to_string(),
                }]);
            }
        };

        // Determine yank type based on current mode
        let yank_type = Self::determine_yank_type(current_mode);

        // Store in yank buffer using YankService
        context
            .services
            .yank
            .yank(selected_text.clone(), yank_type)?;

        // Clear selection directly on ViewModel
        context.view_model.clear_visual_selection()?;

        // Change mode back to Normal
        context.view_model.set_mode(EditorMode::Normal);

        // Prepare events to emit
        let events = vec![
            ModelEvent::TextYanked {
                pane: current_pane,
                text: selected_text.clone(),
                yank_type,
            },
            ModelEvent::ModeChanged {
                old_mode: current_mode,
                new_mode: EditorMode::Normal,
            },
            ModelEvent::SelectionCleared { pane: current_pane },
            ModelEvent::StatusMessageSet {
                message: format!("{} characters yanked", selected_text.len()),
            },
        ];

        Ok(events)
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
        use crate::repl::events::Pane;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let command = YankSelectionCommand::new();

        // Create test context with visual selection
        let context = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
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
        use crate::repl::events::Pane;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let command = YankSelectionCommand::new();

        // Test in Normal mode - should not be relevant
        let context_normal = CommandContext {
            current_mode: EditorMode::Normal,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
        };
        let y_key = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
        assert!(!command.is_relevant(y_key, EditorMode::Normal, &context_normal));

        // Test with no selection - should not be relevant
        let context_no_selection = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: false,
        };
        assert!(!command.is_relevant(y_key, EditorMode::Visual, &context_no_selection));

        // Test in read-only pane - should not be relevant
        let context_readonly = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Response,
            is_read_only: true,
            has_selection: true,
        };
        assert!(!command.is_relevant(y_key, EditorMode::Visual, &context_readonly));

        // Test wrong key - should not be relevant
        let x_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        let context_valid = CommandContext {
            current_mode: EditorMode::Visual,
            current_pane: Pane::Request,
            is_read_only: false,
            has_selection: true,
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
        use crate::repl::services::Services;
        use crate::repl::view_models::ViewModel;

        let command = YankSelectionCommand::new();
        let mut view_model = ViewModel::new();
        let mut services = Services::new();
        let mut context = ExecutionContext {
            view_model: &mut view_model,
            services: &mut services,
        };

        // ViewModel starts in Normal mode by default
        let result = command.handle(&mut context);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("visual modes"));
    }

    #[test]
    fn yank_selection_command_should_emit_events_for_empty_selection() {
        use crate::repl::view_models::ViewModel;

        let _command = YankSelectionCommand::new();
        let _view_model = ViewModel::new();

        // TODO: Set up visual mode and empty selection when we have the methods
        // For now, this test documents the expected behavior

        // This test will be completed when we integrate with the actual ViewModel methods
    }
}
