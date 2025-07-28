//! # View Events
//!
//! Events related to view updates and user input.
//! These events drive UI refreshing and handle user interactions.

use super::types::Pane;
use crossterm::event::KeyEvent;

/// Events emitted when view updates are needed
#[derive(Debug, Clone, PartialEq)]
pub enum ViewEvent {
    /// Full screen redraw required (most expensive - terminal resize, etc)
    FullRedrawRequired,

    /// Specific pane needs full redrawing (scrolling, major content change)
    PaneRedrawRequired { pane: Pane },

    /// Redraw from a specific line to bottom of visible area (for wrapped line edits)
    PartialPaneRedrawRequired {
        pane: Pane,
        start_line: usize, // Logical line number
    },

    /// Status bar needs updating
    StatusBarUpdateRequired,

    /// Only position indicator in status bar needs updating (very cheap)
    PositionIndicatorUpdateRequired,

    /// Only cursor position/style needs updating (cheapest)
    CursorUpdateRequired { pane: Pane },

    /// Scroll position changed
    ScrollChanged {
        pane: Pane,
        old_offset: usize,
        new_offset: usize,
    },
}

/// Input events from user or system
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// Key pressed
    KeyPressed(KeyEvent),

    /// Terminal resized
    TerminalResized { width: u16, height: u16 },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn view_event_full_redraw_should_create() {
        let event = ViewEvent::FullRedrawRequired;
        assert_eq!(event, ViewEvent::FullRedrawRequired);
    }

    #[test]
    fn view_event_pane_redraw_should_carry_pane_data() {
        let event = ViewEvent::PaneRedrawRequired {
            pane: Pane::Request,
        };

        match event {
            ViewEvent::PaneRedrawRequired { pane } => {
                assert_eq!(pane, Pane::Request);
            }
            _ => panic!("Expected PaneRedrawRequired event"),
        }
    }

    #[test]
    fn scroll_changed_event_should_carry_offset_data() {
        let event = ViewEvent::ScrollChanged {
            pane: Pane::Response,
            old_offset: 5,
            new_offset: 10,
        };

        match event {
            ViewEvent::ScrollChanged {
                pane,
                old_offset,
                new_offset,
            } => {
                assert_eq!(pane, Pane::Response);
                assert_eq!(old_offset, 5);
                assert_eq!(new_offset, 10);
            }
            _ => panic!("Expected ScrollChanged event"),
        }
    }

    #[test]
    fn input_event_key_pressed_should_carry_key_data() {
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        let event = InputEvent::KeyPressed(key_event);

        match event {
            InputEvent::KeyPressed(k) => {
                assert_eq!(k.code, KeyCode::Char('a'));
            }
            _ => panic!("Expected KeyPressed event"),
        }
    }

    #[test]
    fn input_event_terminal_resized_should_carry_size_data() {
        let event = InputEvent::TerminalResized {
            width: 80,
            height: 24,
        };

        match event {
            InputEvent::TerminalResized { width, height } => {
                assert_eq!(width, 80);
                assert_eq!(height, 24);
            }
            _ => panic!("Expected TerminalResized event"),
        }
    }
}
