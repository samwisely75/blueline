//! # Settings and Direction Types
//!
//! Shared types for settings and cursor movement directions.

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
    /// Expand tab setting (insert spaces instead of tab)
    ExpandTab,
    /// Cut mode for d/dd/D commands (yank to clipboard)
    DCut,
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
