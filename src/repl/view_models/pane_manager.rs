//! # Pane Management
//!
//! Handles pane switching, mode changes, and pane-related state management.

use crate::repl::events::{EditorMode, LogicalPosition, Pane};
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;

/// Type alias for visual selection state to reduce complexity
type VisualSelectionState = (
    Option<LogicalPosition>,
    Option<LogicalPosition>,
    Option<Pane>,
);

impl ViewModel {
    /// Get current editor mode
    pub fn get_mode(&self) -> EditorMode {
        self.editor.mode()
    }

    /// Get current active pane
    pub fn get_current_pane(&self) -> Pane {
        self.editor.current_pane()
    }

    /// Switch to a different pane
    pub fn switch_pane(&mut self, pane: Pane) -> Result<()> {
        if let Some(_event) = self.editor.set_current_pane(pane) {
            // When switching panes, we need to update cursor and status bar
            self.emit_view_event(crate::repl::events::ViewEvent::StatusBarUpdateRequired);
            self.emit_view_event(crate::repl::events::ViewEvent::CursorUpdateRequired { pane });
        }
        tracing::debug!("Switched to pane: {:?}", pane);
        Ok(())
    }

    /// Change editor mode
    pub fn change_mode(&mut self, mode: EditorMode) -> Result<()> {
        let old_mode = self.editor.mode();
        tracing::debug!("Changing mode from {:?} to {:?}", old_mode, mode);

        let _event = self.editor.set_mode(mode);
        // TODO: self.emit_model_event(event);

        // Clear command buffer when exiting Command mode (e.g., when pressing Escape)
        if old_mode == EditorMode::Command && mode != EditorMode::Command {
            self.ex_command_buffer.clear();
            tracing::debug!("Cleared command buffer when exiting Command mode");
        }

        // Handle visual mode selection state
        if mode == EditorMode::Visual && old_mode != EditorMode::Visual {
            // Entering visual mode - set selection start to current cursor position
            let current_cursor = self.get_cursor_position();
            let current_pane = self.editor.current_pane();
            self.visual_selection_start = Some(current_cursor);
            self.visual_selection_end = Some(current_cursor);
            self.visual_selection_pane = Some(current_pane);
            tracing::info!(
                "Entered visual mode, selection starts at {:?} in {:?}",
                current_cursor,
                current_pane
            );
        } else if old_mode == EditorMode::Visual && mode != EditorMode::Visual {
            // Exiting visual mode - clear selection state
            let current_pane = self.editor.current_pane();
            self.visual_selection_start = None;
            self.visual_selection_end = None;
            self.visual_selection_pane = None;
            tracing::info!("Exited visual mode, cleared selection state");

            // BUGFIX: Emit pane redraw event to clear visual selection highlighting
            // Without this, visual selection highlighting remains on screen after exiting visual mode
            self.emit_view_event(crate::repl::events::ViewEvent::PaneRedrawRequired {
                pane: current_pane,
            });
            tracing::debug!("Emitted pane redraw event to clear visual selection highlighting");
        }

        // Only emit events for what actually needs updating
        self.emit_view_event(crate::repl::events::ViewEvent::StatusBarUpdateRequired);
        self.emit_view_event(crate::repl::events::ViewEvent::CursorUpdateRequired {
            pane: self.editor.current_pane(),
        });

        tracing::info!(
            "Successfully changed mode from {:?} to {:?}",
            old_mode,
            mode
        );
        Ok(())
    }

    /// Get ex command buffer
    pub fn get_ex_command_buffer(&self) -> &str {
        &self.ex_command_buffer
    }

    /// Get visual selection state
    pub fn get_visual_selection(&self) -> VisualSelectionState {
        (
            self.visual_selection_start,
            self.visual_selection_end,
            self.visual_selection_pane,
        )
    }

    /// Check if a position is within visual selection
    pub fn is_position_selected(&self, position: LogicalPosition, pane: Pane) -> bool {
        if let (Some(start), Some(end), Some(selection_pane)) = (
            self.visual_selection_start,
            self.visual_selection_end,
            self.visual_selection_pane,
        ) {
            if pane != selection_pane {
                tracing::trace!(
                    "is_position_selected: pane mismatch - position pane={:?}, selection pane={:?}",
                    pane,
                    selection_pane
                );
                return false;
            }

            // Normalize selection range (start <= end)
            let (normalized_start, normalized_end) = if start.line < end.line
                || (start.line == end.line && start.column <= end.column)
            {
                (start, end)
            } else {
                (end, start)
            };

            tracing::trace!(
                "is_position_selected: checking position={:?} against selection start={:?} end={:?} (normalized: start={:?} end={:?})", 
                position, start, end, normalized_start, normalized_end
            );

            // Check if position is within selection range
            if position.line < normalized_start.line || position.line > normalized_end.line {
                tracing::trace!("is_position_selected: position outside line range");
                return false;
            }

            if position.line == normalized_start.line && position.line == normalized_end.line {
                // Single line selection
                let is_selected = position.column >= normalized_start.column
                    && position.column <= normalized_end.column;
                tracing::trace!(
                    "is_position_selected: single line selection, result={}",
                    is_selected
                );
                return is_selected;
            }

            if position.line == normalized_start.line {
                // First line of multi-line selection
                let is_selected = position.column >= normalized_start.column;
                tracing::trace!(
                    "is_position_selected: first line of multi-line selection, result={}",
                    is_selected
                );
                return is_selected;
            }

            if position.line == normalized_end.line {
                // Last line of multi-line selection
                let is_selected = position.column <= normalized_end.column;
                tracing::trace!(
                    "is_position_selected: last line of multi-line selection, result={}",
                    is_selected
                );
                return is_selected;
            }

            // Middle line of multi-line selection
            tracing::trace!(
                "is_position_selected: middle line of multi-line selection, result=true"
            );
            return true;
        }
        tracing::trace!("is_position_selected: no visual selection active");
        false
    }

    /// Add character to ex command buffer
    pub fn add_ex_command_char(&mut self, ch: char) -> Result<()> {
        self.ex_command_buffer.push(ch);
        self.emit_view_event(crate::repl::events::ViewEvent::StatusBarUpdateRequired);
        Ok(())
    }

    /// Remove last character from ex command buffer
    pub fn backspace_ex_command(&mut self) -> Result<()> {
        self.ex_command_buffer.pop();
        self.emit_view_event(crate::repl::events::ViewEvent::StatusBarUpdateRequired);
        Ok(())
    }

    /// Execute ex command and return resulting command events
    pub fn execute_ex_command(&mut self) -> Result<Vec<crate::repl::commands::CommandEvent>> {
        let command = self.ex_command_buffer.trim();
        let mut events = Vec::new();

        // Handle ex commands
        match command {
            "q" => {
                // Quit the application
                events.push(crate::repl::commands::CommandEvent::QuitRequested);
            }
            "q!" => {
                // Force quit the application
                events.push(crate::repl::commands::CommandEvent::QuitRequested);
            }
            "set wrap" => {
                // Enable word wrap
                if let Err(e) = self.set_wrap_enabled(true) {
                    tracing::warn!("Failed to enable word wrap: {}", e);
                }
            }
            "set nowrap" => {
                // Disable word wrap
                if let Err(e) = self.set_wrap_enabled(false) {
                    tracing::warn!("Failed to disable word wrap: {}", e);
                }
            }
            "show profile" => {
                // Show profile information in status bar
                events.push(crate::repl::commands::CommandEvent::ShowProfileRequested);
            }
            "" => {
                // Empty command, just exit command mode
            }
            _ => {
                // Check if it's a line number command (:<number>)
                if let Ok(line_number) = command.parse::<usize>() {
                    if line_number > 0 {
                        events.push(crate::repl::commands::CommandEvent::CursorMoveRequested {
                            direction: crate::repl::commands::MovementDirection::LineNumber(
                                line_number,
                            ),
                            amount: 1,
                        });
                    } else {
                        tracing::warn!("Invalid line number: {}", line_number);
                    }
                } else {
                    // Unknown command - could emit an error event in future
                    tracing::warn!("Unknown ex command: {}", command);
                }
            }
        }

        // Clear buffer and exit command mode
        self.ex_command_buffer.clear();
        self.change_mode(crate::repl::events::EditorMode::Normal)?;

        Ok(events)
    }

    /// Get request pane height
    pub fn request_pane_height(&self) -> u16 {
        self.request_pane_height
    }

    /// Get response pane height
    pub fn response_pane_height(&self) -> u16 {
        if self.response.status_code().is_some() {
            self.terminal_height
                .saturating_sub(self.request_pane_height)
                .saturating_sub(2) // -2 for separator and status
        } else {
            0
        }
    }
}
