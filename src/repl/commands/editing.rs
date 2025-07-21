//! # Editing Commands
//!
//! This module contains commands for text editing operations like insertion,
//! deletion, and mode switching.

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::repl::{
    command::{Command, CommandResult},
    model::{AppState, EditorMode, Pane},
};

/// Command for entering insert mode (i, I, A)
pub struct EnterInsertModeCommand;

impl EnterInsertModeCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for EnterInsertModeCommand {
    fn is_relevant(&self, state: &AppState) -> bool {
        // Only relevant in Normal mode, Request pane
        matches!(state.mode, EditorMode::Normal) && state.current_pane == Pane::Request
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Check if the key matches what we handle
        if event.modifiers != KeyModifiers::NONE {
            return Ok(false);
        }

        match event.code {
            KeyCode::Char('i') => {
                // Insert at current position
                state.mode = EditorMode::Insert;
                state.status_message = "-- INSERT --".to_string();
                Ok(true)
            }
            KeyCode::Char('I') => {
                // Insert at beginning of line
                state.request_buffer.cursor_col = 0;
                state.mode = EditorMode::Insert;
                state.status_message = "-- INSERT --".to_string();
                Ok(true)
            }
            KeyCode::Char('A') => {
                // Append at end of line
                if let Some(line) = state
                    .request_buffer
                    .lines
                    .get(state.request_buffer.cursor_line)
                {
                    state.request_buffer.cursor_col = line.len();
                }
                state.mode = EditorMode::Insert;
                state.status_message = "-- INSERT --".to_string();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn name(&self) -> &'static str {
        "EnterInsertMode"
    }
}

/// Command for exiting insert mode (Esc)
pub struct ExitInsertModeCommand;

impl ExitInsertModeCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for ExitInsertModeCommand {
    fn is_relevant(&self, state: &AppState) -> bool {
        // Only relevant in Insert mode
        matches!(state.mode, EditorMode::Insert)
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        if matches!(event.code, KeyCode::Esc) {
            state.mode = EditorMode::Normal;
            state.status_message = "".to_string();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "ExitInsertMode"
    }
}

/// Command for inserting characters (any printable character in insert mode)
pub struct InsertCharCommand;

impl InsertCharCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for InsertCharCommand {
    fn is_relevant(&self, state: &AppState) -> bool {
        // Only relevant in Insert mode, Request pane
        matches!(state.mode, EditorMode::Insert) && state.current_pane == Pane::Request
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        // Check for printable characters (no control modifiers)
        if let KeyCode::Char(ch) = event.code {
            if !event.modifiers.contains(KeyModifiers::CONTROL) {
                // Ensure we have a valid line to insert into
                if state.request_buffer.cursor_line >= state.request_buffer.lines.len() {
                    state.request_buffer.lines.push(String::new());
                }

                let line = &mut state.request_buffer.lines[state.request_buffer.cursor_line];
                if state.request_buffer.cursor_col <= line.len() {
                    line.insert(state.request_buffer.cursor_col, ch);
                    state.request_buffer.cursor_col += 1;
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn name(&self) -> &'static str {
        "InsertChar"
    }
}

/// Command for inserting new lines (Enter in insert mode)
pub struct InsertNewLineCommand;

impl InsertNewLineCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for InsertNewLineCommand {
    fn is_relevant(&self, state: &AppState) -> bool {
        // Only relevant in Insert mode, Request pane
        matches!(state.mode, EditorMode::Insert) && state.current_pane == Pane::Request
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        if matches!(event.code, KeyCode::Enter) {
            // Ensure we have a valid line
            if state.request_buffer.cursor_line >= state.request_buffer.lines.len() {
                state.request_buffer.lines.push(String::new());
            }

            let line = &mut state.request_buffer.lines[state.request_buffer.cursor_line];
            let remainder = line.split_off(state.request_buffer.cursor_col);
            state.request_buffer.cursor_line += 1;
            state
                .request_buffer
                .lines
                .insert(state.request_buffer.cursor_line, remainder);
            state.request_buffer.cursor_col = 0;

            // Auto-scroll down if cursor goes below visible area
            let visible_height = state.get_request_pane_height();
            if state.request_buffer.cursor_line
                >= state.request_buffer.scroll_offset + visible_height
            {
                state.request_buffer.scroll_offset =
                    state.request_buffer.cursor_line - visible_height + 1;
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "InsertNewLine"
    }
}

/// Command for deleting characters (Backspace in insert mode)
pub struct DeleteCharCommand;

impl DeleteCharCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for DeleteCharCommand {
    fn is_relevant(&self, state: &AppState) -> bool {
        // Only relevant in Insert mode, Request pane
        matches!(state.mode, EditorMode::Insert) && state.current_pane == Pane::Request
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        if matches!(event.code, KeyCode::Backspace) {
            if state.request_buffer.cursor_line >= state.request_buffer.lines.len() {
                return Ok(false);
            }

            let line = &mut state.request_buffer.lines[state.request_buffer.cursor_line];
            if state.request_buffer.cursor_col > 0 && state.request_buffer.cursor_col <= line.len()
            {
                line.remove(state.request_buffer.cursor_col - 1);
                state.request_buffer.cursor_col -= 1;
                Ok(true)
            } else if state.request_buffer.cursor_col == 0 && state.request_buffer.cursor_line > 0 {
                // At beginning of line, join with previous line
                let current_line = state
                    .request_buffer
                    .lines
                    .remove(state.request_buffer.cursor_line);
                state.request_buffer.cursor_line -= 1;
                state.request_buffer.cursor_col =
                    state.request_buffer.lines[state.request_buffer.cursor_line].len();
                state.request_buffer.lines[state.request_buffer.cursor_line]
                    .push_str(&current_line);
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "DeleteChar"
    }
}

/// Command for entering command mode (:)
pub struct EnterCommandModeCommand;

impl EnterCommandModeCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for EnterCommandModeCommand {
    fn is_relevant(&self, state: &AppState) -> bool {
        // Only relevant in Normal mode
        matches!(state.mode, EditorMode::Normal)
    }

    fn process(&self, event: KeyEvent, state: &mut AppState) -> Result<bool> {
        if matches!(event.code, KeyCode::Char(':')) && event.modifiers == KeyModifiers::NONE {
            state.mode = EditorMode::Command;
            state.command_buffer.clear();
            state.status_message = ":".to_string();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn name(&self) -> &'static str {
        "EnterCommandMode"
    }
}
