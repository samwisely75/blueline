//! # View Events
//!
//! Events related to view updates and user input.
//! These events drive UI refreshing and handle user interactions.

// Pane import removed - no longer needed for abstracted events
use crossterm::event::KeyEvent;

/// Events emitted when view updates are needed
/// These events are completely abstracted - external components never need to know about specific panes
#[derive(Debug, Clone, PartialEq)]
pub enum ViewEvent {
    /// Full screen redraw required (most expensive - terminal resize, etc)
    FullRedrawRequired,

    /// Current active area needs full redrawing (scrolling, major content change)
    CurrentAreaRedrawRequired,

    /// Secondary/display area needs full redrawing
    SecondaryAreaRedrawRequired,

    /// Redraw current area from a specific line to bottom of visible area
    CurrentAreaPartialRedrawRequired {
        start_line: usize, // Logical line number
    },

    /// Redraw secondary area from a specific line to bottom of visible area
    SecondaryAreaPartialRedrawRequired {
        start_line: usize, // Logical line number
    },

    /// Status bar needs updating
    StatusBarUpdateRequired,

    /// Only position indicator in status bar needs updating (very cheap)
    PositionIndicatorUpdateRequired,

    /// Active cursor position/style needs updating (cheapest)
    ActiveCursorUpdateRequired,

    /// Scroll position changed in current area
    CurrentAreaScrollChanged {
        old_offset: usize,
        new_offset: usize,
    },

    /// Scroll position changed in secondary area
    SecondaryAreaScrollChanged {
        old_offset: usize,
        new_offset: usize,
    },

    /// Content area focus switched (for cursor style, highlighting, etc)
    FocusSwitched,

    // Domain-specific events for clearer semantics
    /// Request content has been modified
    RequestContentChanged,

    /// Response content needs to be displayed
    ResponseContentChanged,

    /// Both request and response areas need redraw (for layout changes)
    AllContentAreasRedrawRequired,
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
    fn view_event_current_area_redraw_should_create() {
        let event = ViewEvent::CurrentAreaRedrawRequired;
        assert_eq!(event, ViewEvent::CurrentAreaRedrawRequired);
    }

    #[test]
    fn scroll_changed_event_should_carry_offset_data() {
        let event = ViewEvent::CurrentAreaScrollChanged {
            old_offset: 5,
            new_offset: 10,
        };

        match event {
            ViewEvent::CurrentAreaScrollChanged {
                old_offset,
                new_offset,
            } => {
                assert_eq!(old_offset, 5);
                assert_eq!(new_offset, 10);
            }
            _ => panic!("Expected CurrentAreaScrollChanged event"),
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
