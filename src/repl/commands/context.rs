//! # Command Context
//!
//! Context and service access for commands.
//! Uses trait-based access to provide type-safe, minimal exposure to services.

use crate::repl::events::{EditorMode, LogicalPosition, Pane};
use crate::repl::view_models::ViewModel;
use bluenote::HttpClient;

/// Read-only snapshot of ViewModel state for commands
#[derive(Debug, Clone)]
pub struct ViewModelSnapshot {
    pub current_mode: EditorMode,
    pub current_pane: Pane,
    pub cursor_position: LogicalPosition,
    pub request_text: String,
    pub response_text: String,
    pub terminal_dimensions: (u16, u16),
}

impl ViewModelSnapshot {
    /// Create snapshot from current ViewModel state
    pub fn from_view_model(view_model: &ViewModel) -> Self {
        Self {
            current_mode: view_model.get_mode(),
            current_pane: view_model.get_current_pane(),
            cursor_position: view_model.get_cursor_position(),
            request_text: view_model.get_request_text(),
            response_text: view_model.get_response_text(),
            terminal_dimensions: view_model.terminal_size(),
        }
    }
}

/// Base context available to all commands
pub struct CommandContext {
    pub state: ViewModelSnapshot,
}

impl CommandContext {
    pub fn new(state: ViewModelSnapshot) -> Self {
        Self { state }
    }
}

/// Service access traits for type-safe dependency injection
///
/// Access to HTTP client for commands that need to make HTTP requests
pub trait HttpClientAccess {
    fn http_client(&self) -> Option<&HttpClient>;
}

/// Access to terminal information for commands that need display info
pub trait TerminalAccess {
    fn terminal_size(&self) -> (u16, u16);
}

/// Access to buffer information for commands that need text data
pub trait BufferAccess {
    fn get_buffer_content(&self, pane: Pane) -> &str;
    fn get_line_count(&self, pane: Pane) -> usize;
    fn get_line_length(&self, pane: Pane, line: usize) -> usize;
}

/// Extended context that includes HTTP client access
pub struct HttpCommandContext {
    pub base: CommandContext,
    pub http_client: Option<HttpClient>,
}

impl HttpCommandContext {
    pub fn new(state: ViewModelSnapshot, http_client: Option<HttpClient>) -> Self {
        Self {
            base: CommandContext::new(state),
            http_client,
        }
    }

    /// Get base context for state access
    pub fn context(&self) -> &CommandContext {
        &self.base
    }

    /// Get state snapshot
    pub fn state(&self) -> &ViewModelSnapshot {
        &self.base.state
    }
}

impl HttpClientAccess for HttpCommandContext {
    fn http_client(&self) -> Option<&HttpClient> {
        self.http_client.as_ref()
    }
}

impl TerminalAccess for CommandContext {
    fn terminal_size(&self) -> (u16, u16) {
        self.state.terminal_dimensions
    }
}

impl TerminalAccess for HttpCommandContext {
    fn terminal_size(&self) -> (u16, u16) {
        self.base.terminal_size()
    }
}

// Implement basic access for both context types
impl AsRef<CommandContext> for CommandContext {
    fn as_ref(&self) -> &CommandContext {
        self
    }
}

impl AsRef<CommandContext> for HttpCommandContext {
    fn as_ref(&self) -> &CommandContext {
        &self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::view_models::ViewModel;

    #[test]
    fn view_model_snapshot_should_capture_state() {
        let view_model = ViewModel::new();
        let snapshot = ViewModelSnapshot::from_view_model(&view_model);

        assert_eq!(snapshot.current_mode, EditorMode::Normal);
        assert_eq!(snapshot.current_pane, Pane::Request);
        assert_eq!(snapshot.cursor_position, LogicalPosition::zero());
    }

    #[test]
    fn command_context_should_provide_state() {
        let view_model = ViewModel::new();
        let snapshot = ViewModelSnapshot::from_view_model(&view_model);
        let context = CommandContext::new(snapshot);

        assert_eq!(context.state.current_mode, EditorMode::Normal);
    }

    #[test]
    fn http_command_context_should_provide_http_access() {
        let view_model = ViewModel::new();
        let snapshot = ViewModelSnapshot::from_view_model(&view_model);
        let context = HttpCommandContext::new(snapshot, None);

        assert!(context.http_client().is_none());
    }

    #[test]
    fn terminal_access_should_provide_size() {
        let view_model = ViewModel::new();
        let snapshot = ViewModelSnapshot::from_view_model(&view_model);
        let context = CommandContext::new(snapshot);

        let (width, height) = context.terminal_size();
        assert_eq!(width, 80); // Default terminal width
        assert_eq!(height, 24); // Default terminal height
    }
}
