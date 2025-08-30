//! # Command Context
//!
//! Context and service access for commands.
//! Uses trait-based access to provide type-safe, minimal exposure to services.

use crate::repl::models::pane_state::{EditorMode, LogicalPosition, Pane};
use crate::repl::models::AppState;
use bluenote::HttpClient;

/// Read-only snapshot of AppState for commands
#[derive(Debug, Clone)]
pub struct AppStateSnapshot {
    pub current_mode: EditorMode,
    pub current_pane: Pane,
    pub cursor_position: LogicalPosition,
    pub request_text: String,
    pub response_text: String,
    pub terminal_dimensions: (u16, u16),
    pub expand_tab: bool,
    pub tab_width: usize,
}

impl AppStateSnapshot {
    /// Create snapshot from current AppState state
    pub fn from_app_state(app_state: &AppState) -> Self {
        Self {
            current_mode: app_state.get_mode(),
            current_pane: app_state.get_current_pane(),
            cursor_position: app_state.get_cursor_position(),
            request_text: app_state.get_request_text(),
            response_text: app_state.get_response_text(),
            terminal_dimensions: app_state.terminal_size(),
            expand_tab: app_state.pane_manager().get_expand_tab(),
            tab_width: app_state.pane_manager().get_tab_width(),
        }
    }
}

/// Base context available to all commands
pub struct CommandContext {
    pub state: AppStateSnapshot,
}

impl CommandContext {
    pub fn new(state: AppStateSnapshot) -> Self {
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
    pub fn new(state: AppStateSnapshot, http_client: Option<HttpClient>) -> Self {
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
    pub fn state(&self) -> &AppStateSnapshot {
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
    use crate::repl::models::AppState;

    #[test]
    fn app_state_snapshot_should_capture_state() {
        let app_state = AppState::new();
        let snapshot = AppStateSnapshot::from_app_state(&app_state);

        assert_eq!(snapshot.current_mode, EditorMode::Normal);
        assert_eq!(snapshot.current_pane, Pane::Request);
        assert_eq!(snapshot.cursor_position, LogicalPosition::zero());
    }

    #[test]
    fn command_context_should_provide_state() {
        let app_state = AppState::new();
        let snapshot = AppStateSnapshot::from_app_state(&app_state);
        let context = CommandContext::new(snapshot);

        assert_eq!(context.state.current_mode, EditorMode::Normal);
    }

    #[test]
    fn http_command_context_should_provide_http_access() {
        let app_state = AppState::new();
        let snapshot = AppStateSnapshot::from_app_state(&app_state);
        let context = HttpCommandContext::new(snapshot, None);

        assert!(context.http_client().is_none());
    }

    #[test]
    fn terminal_access_should_provide_size() {
        let app_state = AppState::new();
        let snapshot = AppStateSnapshot::from_app_state(&app_state);
        let context = CommandContext::new(snapshot);

        let (width, height) = context.terminal_size();
        assert_eq!(width, 80); // Default terminal width
        assert_eq!(height, 24); // Default terminal height
    }
}
