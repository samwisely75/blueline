//! # Model Events
//!
//! Semantic events that describe state changes in the application.
//! These events are produced by Commands and describe WHAT happened,
//! not HOW to display it (that's handled by RenderEvents later).
//!
//! Model Events represent business logic changes and maintain the separation
//! between business logic (Commands) and display logic (ViewRenderer).

use crate::repl::events::{EditorMode, LogicalPosition, Pane};

/// Semantic events describing state changes in the application
///
/// These events focus on WHAT changed rather than HOW to display it.
/// Each event represents a meaningful business logic state change
/// that may trigger rendering updates later.
#[derive(Debug, Clone, PartialEq)]
pub enum ModelEvent {
    /// Text was inserted at a specific position
    TextInserted {
        pane: Pane,
        position: LogicalPosition,
        text: String,
    },

    /// Text was deleted from a range
    TextDeleted {
        pane: Pane,
        position: LogicalPosition,
        deleted_text: String,
    },

    /// Cursor position changed
    CursorMoved {
        pane: Pane,
        old_position: LogicalPosition,
        new_position: LogicalPosition,
    },

    /// Visual selection state changed
    SelectionChanged {
        pane: Pane,
        start: Option<LogicalPosition>,
        end: Option<LogicalPosition>,
    },

    /// Visual selection was cleared
    SelectionCleared { pane: Pane },

    /// Editor mode changed
    ModeChanged {
        old_mode: EditorMode,
        new_mode: EditorMode,
    },

    /// Active pane switched
    PaneSwitched { from: Pane, to: Pane },

    /// Status message was set
    StatusMessageSet { message: String },

    /// Status message was cleared
    StatusMessageCleared,

    /// Text was yanked to buffer
    TextYanked {
        pane: Pane,
        text: String,
        yank_type: YankType,
    },

    /// HTTP request was initiated
    HttpRequestStarted { method: String, url: String },

    /// HTTP response was received
    HttpResponseReceived { status: u16, body: String },
}

/// Type of yank operation (from existing YankType)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YankType {
    /// Character-wise yank (vim's 'v' mode)
    Character,
    /// Line-wise yank (vim's 'V' mode)  
    Line,
    /// Block-wise yank (vim's Ctrl+V mode)
    Block,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_event_text_inserted_should_contain_position_and_text() {
        let event = ModelEvent::TextInserted {
            pane: Pane::Request,
            position: LogicalPosition::new(1, 5),
            text: "hello".to_string(),
        };

        match event {
            ModelEvent::TextInserted {
                pane,
                position,
                text,
            } => {
                assert_eq!(pane, Pane::Request);
                assert_eq!(position.line, 1);
                assert_eq!(position.column, 5);
                assert_eq!(text, "hello");
            }
            _ => panic!("Expected TextInserted event"),
        }
    }

    #[test]
    fn model_event_mode_changed_should_track_old_and_new_modes() {
        let event = ModelEvent::ModeChanged {
            old_mode: EditorMode::Normal,
            new_mode: EditorMode::Insert,
        };

        match event {
            ModelEvent::ModeChanged { old_mode, new_mode } => {
                assert_eq!(old_mode, EditorMode::Normal);
                assert_eq!(new_mode, EditorMode::Insert);
            }
            _ => panic!("Expected ModeChanged event"),
        }
    }

    #[test]
    fn model_event_selection_changed_should_handle_none_values() {
        let event = ModelEvent::SelectionChanged {
            pane: Pane::Response,
            start: None,
            end: None,
        };

        match event {
            ModelEvent::SelectionChanged { pane, start, end } => {
                assert_eq!(pane, Pane::Response);
                assert!(start.is_none());
                assert!(end.is_none());
            }
            _ => panic!("Expected SelectionChanged event"),
        }
    }

    #[test]
    fn model_event_text_yanked_should_include_yank_type() {
        let event = ModelEvent::TextYanked {
            pane: Pane::Request,
            text: "hello world".to_string(),
            yank_type: YankType::Line,
        };

        match event {
            ModelEvent::TextYanked {
                pane,
                text,
                yank_type,
            } => {
                assert_eq!(pane, Pane::Request);
                assert_eq!(text, "hello world");
                assert_eq!(yank_type, YankType::Line);
            }
            _ => panic!("Expected TextYanked event"),
        }
    }

    #[test]
    fn yank_type_enum_should_support_all_visual_modes() {
        let character = YankType::Character;
        let line = YankType::Line;
        let block = YankType::Block;

        // Verify all types are distinct
        assert_ne!(character, line);
        assert_ne!(line, block);
        assert_ne!(character, block);
    }
}
