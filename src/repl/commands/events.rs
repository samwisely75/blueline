//! # Command Events
//!
//! Events produced by commands that describe what should happen.
//! Commands produce these events, and the controller applies them to the ViewModel.
//! This maintains proper separation of concerns - commands suggest, controller decides.

use crate::repl::events::{EditorMode, LogicalPosition, Pane};

/// Type alias for HTTP headers to reduce complexity
pub type HttpHeaders = Vec<(String, String)>;

/// Available settings that can be changed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    /// Line wrapping setting
    Wrap,
    /// Line numbers display setting
    LineNumbers,
    /// System clipboard integration
    Clipboard,
    /// Tab stop width
    TabStop,
}

/// Values for settings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingValue {
    /// Enable the setting
    On,
    /// Disable the setting
    Off,
    /// Numeric value for the setting
    Number(usize),
}

/// Events that commands can produce to request changes
#[derive(Debug, Clone, PartialEq)]
pub enum CommandEvent {
    /// Request cursor movement
    CursorMoveRequested {
        direction: MovementDirection,
        amount: usize,
    },

    /// Request cursor position change
    CursorPositionRequested { position: LogicalPosition },

    /// Request text insertion
    TextInsertRequested {
        text: String,
        position: LogicalPosition,
    },

    /// Request text deletion
    TextDeleteRequested {
        position: LogicalPosition,
        amount: usize,
        direction: MovementDirection,
    },

    /// Request mode change
    ModeChangeRequested { new_mode: EditorMode },

    /// Request to restore previous mode (for command cancellation)
    RestorePreviousModeRequested,

    /// Request pane switch
    PaneSwitchRequested { target_pane: Pane },

    /// Request HTTP execution
    HttpRequestRequested {
        method: String,
        url: String,
        headers: HttpHeaders,
        body: Option<String>,
    },

    /// Request terminal size update
    TerminalResizeRequested { width: u16, height: u16 },

    /// Request to quit application
    QuitRequested,

    /// Request to add character to ex command buffer
    ExCommandCharRequested { ch: char },

    /// Request to backspace in ex command buffer
    ExCommandBackspaceRequested,

    /// Request to execute ex command in buffer
    ExCommandExecuteRequested,

    /// Request to show profile information in status bar
    ShowProfileRequested,

    /// Request to change a setting (wrap, line numbers, etc.)
    SettingChangeRequested {
        setting: Setting,
        value: SettingValue,
    },

    /// Request to yank (copy) selected text to yank buffer
    YankSelectionRequested,

    /// Request to paste yanked text after cursor
    PasteAfterRequested,

    /// Request to paste yanked text at current cursor position
    PasteAtCursorRequested,

    /// No action needed (for commands that only query state)
    NoAction,
}

/// Direction for movement operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementDirection {
    Left,
    Right,
    Up,
    Down,
    LineStart,
    LineEnd,
    LineEndForAppend, // Special case for 'A' command - positions AFTER last character
    DocumentStart,
    DocumentEnd,
    WordForward,
    WordBackward,
    WordEnd,
    ScrollLeft,
    ScrollRight,
    /// Full page down (Ctrl+f)
    PageDown,
    /// Full page up (Ctrl+b)
    PageUp,
    /// Half page down (Ctrl+d)
    HalfPageDown,
    /// Half page up (Ctrl+u)
    HalfPageUp,
    /// Move to a specific line number (1-based)
    LineNumber(usize),
}

impl CommandEvent {
    /// Create a simple cursor move event
    pub fn cursor_move(direction: MovementDirection) -> Self {
        Self::CursorMoveRequested {
            direction,
            amount: 1,
        }
    }

    /// Create a cursor move event with specific amount
    pub fn cursor_move_by(direction: MovementDirection, amount: usize) -> Self {
        Self::CursorMoveRequested { direction, amount }
    }

    /// Create a mode change event
    pub fn mode_change(new_mode: EditorMode) -> Self {
        Self::ModeChangeRequested { new_mode }
    }

    /// Create a restore previous mode event
    pub fn restore_previous_mode() -> Self {
        Self::RestorePreviousModeRequested
    }

    /// Create a pane switch event
    pub fn pane_switch(target_pane: Pane) -> Self {
        Self::PaneSwitchRequested { target_pane }
    }

    /// Create a text insert event
    pub fn text_insert(text: String, position: LogicalPosition) -> Self {
        Self::TextInsertRequested { text, position }
    }

    /// Create an HTTP request event
    pub fn http_request(method: String, url: String, body: Option<String>) -> Self {
        Self::HttpRequestRequested {
            method,
            url,
            headers: Vec::new(),
            body,
        }
    }

    /// Create an HTTP request event with headers
    pub fn http_request_with_headers(
        method: String,
        url: String,
        headers: HttpHeaders,
        body: Option<String>,
    ) -> Self {
        Self::HttpRequestRequested {
            method,
            url,
            headers,
            body,
        }
    }

    /// Create a yank selection event
    pub fn yank_selection() -> Self {
        Self::YankSelectionRequested
    }

    /// Create a paste after event
    pub fn paste_after() -> Self {
        Self::PasteAfterRequested
    }

    /// Create a paste at cursor event
    pub fn paste_at_cursor() -> Self {
        Self::PasteAtCursorRequested
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_event_should_create_cursor_move() {
        let event = CommandEvent::cursor_move(MovementDirection::Left);
        assert_eq!(
            event,
            CommandEvent::CursorMoveRequested {
                direction: MovementDirection::Left,
                amount: 1
            }
        );
    }

    #[test]
    fn command_event_should_create_mode_change() {
        let event = CommandEvent::mode_change(EditorMode::Insert);
        assert_eq!(
            event,
            CommandEvent::ModeChangeRequested {
                new_mode: EditorMode::Insert
            }
        );
    }

    #[test]
    fn command_event_should_create_restore_previous_mode() {
        let event = CommandEvent::restore_previous_mode();
        assert_eq!(event, CommandEvent::RestorePreviousModeRequested);
    }

    #[test]
    fn command_event_should_create_http_request() {
        let event =
            CommandEvent::http_request("GET".to_string(), "http://example.com".to_string(), None);
        assert_eq!(
            event,
            CommandEvent::HttpRequestRequested {
                method: "GET".to_string(),
                url: "http://example.com".to_string(),
                headers: Vec::new(),
                body: None,
            }
        );
    }
}
