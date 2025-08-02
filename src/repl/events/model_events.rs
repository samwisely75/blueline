//! # Model Events
//!
//! Events emitted when data models change state.
//! These events notify the system of data changes for reactive updates.

use super::types::{EditorMode, LogicalPosition, LogicalRange, Pane};

/// Events emitted when models change
#[derive(Debug, Clone, PartialEq)]
pub enum ModelEvent {
    /// Cursor moved in a pane
    CursorMoved {
        pane: Pane,
        old_pos: LogicalPosition,
        new_pos: LogicalPosition,
    },

    /// Text was inserted
    TextInserted {
        pane: Pane,
        position: LogicalPosition,
        text: String,
    },

    /// Text was deleted
    TextDeleted { pane: Pane, range: LogicalRange },

    /// Editor mode changed
    ModeChanged {
        old_mode: EditorMode,
        new_mode: EditorMode,
    },

    /// Active pane switched
    PaneSwitched { old_pane: Pane, new_pane: Pane },

    /// HTTP request was executed
    RequestExecuted { method: String, url: String },

    /// HTTP response received
    ResponseReceived { status_code: u16, body: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_moved_event_should_carry_position_data() {
        let old_pos = LogicalPosition::new(1, 2);
        let new_pos = LogicalPosition::new(3, 4);
        let event = ModelEvent::CursorMoved {
            pane: Pane::Request,
            old_pos,
            new_pos,
        };

        match event {
            ModelEvent::CursorMoved {
                pane,
                old_pos: o,
                new_pos: n,
            } => {
                assert_eq!(pane, Pane::Request);
                assert_eq!(o, old_pos);
                assert_eq!(n, new_pos);
            }
            _ => panic!("Expected CursorMoved event"),
        }
    }

    #[test]
    fn text_inserted_event_should_carry_text_data() {
        let event = ModelEvent::TextInserted {
            pane: Pane::Request,
            position: LogicalPosition::zero(),
            text: "hello".to_string(),
        };

        match event {
            ModelEvent::TextInserted { text, .. } => {
                assert_eq!(text, "hello");
            }
            _ => panic!("Expected TextInserted event"),
        }
    }

    #[test]
    fn mode_changed_event_should_carry_mode_data() {
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
}
