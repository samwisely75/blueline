//! # Editor Model
//!
//! Core editor state including mode and current pane.
//! Manages high-level editor state without UI concerns.

use crate::repl::events::{EditorMode, ModelEvent, Pane};

/// Editor state model
#[derive(Debug, Clone)]
pub struct EditorModel {
    mode: EditorMode,
    current_pane: Pane,
}

impl EditorModel {
    /// Create new editor in normal mode
    pub fn new() -> Self {
        Self {
            mode: EditorMode::Normal,
            current_pane: Pane::Request,
        }
    }

    /// Get current mode
    pub fn mode(&self) -> EditorMode {
        self.mode
    }

    /// Set mode, returning event if changed
    pub fn set_mode(&mut self, new_mode: EditorMode) -> Option<ModelEvent> {
        if self.mode != new_mode {
            let old_mode = self.mode;
            self.mode = new_mode;
            Some(ModelEvent::ModeChanged { old_mode, new_mode })
        } else {
            None
        }
    }

    /// Get current pane
    pub fn current_pane(&self) -> Pane {
        self.current_pane
    }

    /// Set current pane, returning event if changed
    pub fn set_current_pane(&mut self, new_pane: Pane) -> Option<ModelEvent> {
        if self.current_pane != new_pane {
            let old_pane = self.current_pane;
            self.current_pane = new_pane;
            Some(ModelEvent::PaneSwitched { old_pane, new_pane })
        } else {
            None
        }
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
    fn editor_model_should_change_mode() {
        let mut editor = EditorModel::new();

        let event = editor.set_mode(EditorMode::Insert);

        assert!(event.is_some());
        assert_eq!(editor.mode(), EditorMode::Insert);
    }

    #[test]
    fn editor_model_should_switch_pane() {
        let mut editor = EditorModel::new();

        let event = editor.set_current_pane(Pane::Response);

        assert!(event.is_some());
        assert_eq!(editor.current_pane(), Pane::Response);
    }
}
