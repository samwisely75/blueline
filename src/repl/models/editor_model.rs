//! Editor model for MVVM architecture
//!
//! This model manages editor state like cursor position, mode, and active pane.
//! It contains only the logical editor state without any view concerns.

use crate::repl::events::{LogicalPosition, ModelEvent};
use crate::repl::model::{EditorMode, Pane};

/// Editor model containing cursor positions, mode, and pane state
///
/// This model manages the fundamental editor state that drives the editing experience.
/// It emits events when state changes occur, allowing other components to react accordingly.
#[derive(Debug, Clone)]
pub struct EditorModel {
    /// Logical cursor position in request pane
    pub request_cursor: LogicalPosition,
    /// Logical cursor position in response pane
    pub response_cursor: LogicalPosition,
    /// Currently active pane
    pub current_pane: Pane,
    /// Current editor mode
    pub mode: EditorMode,
    /// Command buffer for command mode
    pub command_buffer: String,
}

impl EditorModel {
    /// Create a new editor model with default state
    pub fn new() -> Self {
        Self {
            request_cursor: LogicalPosition { line: 0, column: 0 },
            response_cursor: LogicalPosition { line: 0, column: 0 },
            current_pane: Pane::Request,
            mode: EditorMode::Normal,
            command_buffer: String::new(),
        }
    }
    
    /// Get cursor position for the specified pane
    pub fn get_cursor(&self, pane: Pane) -> LogicalPosition {
        match pane {
            Pane::Request => self.request_cursor,
            Pane::Response => self.response_cursor,
        }
    }
    
    /// Set cursor position for the specified pane
    /// Returns event if cursor position changed
    pub fn set_cursor(&mut self, pane: Pane, new_pos: LogicalPosition) -> Option<ModelEvent> {
        let cursor = match pane {
            Pane::Request => &mut self.request_cursor,
            Pane::Response => &mut self.response_cursor,
        };
        
        if *cursor != new_pos {
            let old_pos = *cursor;
            *cursor = new_pos;
            Some(ModelEvent::CursorMoved {
                pane,
                old_pos,
                new_pos,
            })
        } else {
            None
        }
    }
    
    /// Switch to different pane
    pub fn switch_pane(&mut self, new_pane: Pane) -> Option<ModelEvent> {
        if self.current_pane != new_pane {
            let old_pane = self.current_pane;
            self.current_pane = new_pane;
            Some(ModelEvent::PaneSwitched {
                from: old_pane,
                to: new_pane,
            })
        } else {
            None
        }
    }
    
    /// Change editor mode
    pub fn change_mode(&mut self, new_mode: EditorMode) -> Option<ModelEvent> {
        if self.mode != new_mode {
            let old_mode = self.mode.clone();
            self.mode = new_mode.clone();
            Some(ModelEvent::ModeChanged {
                from: old_mode,
                to: new_mode,
            })
        } else {
            None
        }
    }
    
    /// Update command buffer (for command mode)
    pub fn set_command_buffer(&mut self, buffer: String) {
        self.command_buffer = buffer;
    }
    
    /// Clear command buffer
    pub fn clear_command_buffer(&mut self) {
        self.command_buffer.clear();
    }
}

impl Default for EditorModel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_model_should_create_with_default_state() {
        let model = EditorModel::new();
        
        assert_eq!(model.current_pane, Pane::Request);
        assert_eq!(model.mode, EditorMode::Normal);
        assert_eq!(model.request_cursor, LogicalPosition { line: 0, column: 0 });
        assert_eq!(model.response_cursor, LogicalPosition { line: 0, column: 0 });
        assert_eq!(model.command_buffer, "");
    }

    #[test]
    fn set_cursor_should_emit_event_when_position_changes() {
        let mut model = EditorModel::new();
        let new_pos = LogicalPosition { line: 1, column: 5 };
        
        let event = model.set_cursor(Pane::Request, new_pos).unwrap();
        
        assert_eq!(model.request_cursor, new_pos);
        match event {
            ModelEvent::CursorMoved { pane, old_pos, new_pos: event_new_pos } => {
                assert_eq!(pane, Pane::Request);
                assert_eq!(old_pos, LogicalPosition { line: 0, column: 0 });
                assert_eq!(event_new_pos, new_pos);
            }
            _ => panic!("Expected CursorMoved event"),
        }
    }

    #[test]
    fn set_cursor_should_return_none_when_position_unchanged() {
        let mut model = EditorModel::new();
        let same_pos = LogicalPosition { line: 0, column: 0 };
        
        let event = model.set_cursor(Pane::Request, same_pos);
        
        assert!(event.is_none());
        assert_eq!(model.request_cursor, same_pos);
    }

    #[test]
    fn switch_pane_should_emit_event() {
        let mut model = EditorModel::new();
        
        let event = model.switch_pane(Pane::Response).unwrap();
        
        assert_eq!(model.current_pane, Pane::Response);
        match event {
            ModelEvent::PaneSwitched { from, to } => {
                assert_eq!(from, Pane::Request);
                assert_eq!(to, Pane::Response);
            }
            _ => panic!("Expected PaneSwitched event"),
        }
    }

    #[test]
    fn switch_pane_should_return_none_when_same() {
        let mut model = EditorModel::new();
        
        let event = model.switch_pane(Pane::Request);
        
        assert!(event.is_none());
        assert_eq!(model.current_pane, Pane::Request);
    }

    #[test]
    fn change_mode_should_emit_event() {
        let mut model = EditorModel::new();
        
        let event = model.change_mode(EditorMode::Insert).unwrap();
        
        assert_eq!(model.mode, EditorMode::Insert);
        match event {
            ModelEvent::ModeChanged { from, to } => {
                assert_eq!(from, EditorMode::Normal);
                assert_eq!(to, EditorMode::Insert);
            }
            _ => panic!("Expected ModeChanged event"),
        }
    }

    #[test]
    fn change_mode_should_return_none_when_same() {
        let mut model = EditorModel::new();
        
        let event = model.change_mode(EditorMode::Normal);
        
        assert!(event.is_none());
        assert_eq!(model.mode, EditorMode::Normal);
    }

    #[test]
    fn command_buffer_operations_should_work() {
        let mut model = EditorModel::new();
        
        model.set_command_buffer("test command".to_string());
        assert_eq!(model.command_buffer, "test command");
        
        model.clear_command_buffer();
        assert_eq!(model.command_buffer, "");
    }
}