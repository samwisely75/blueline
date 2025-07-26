//! # View Trait - Abstraction for ViewManager to enable mocking
//!
//! This module defines the trait that abstracts the view rendering operations,
//! allowing for dependency injection of mock implementations during testing.

use super::model::AppState;
use anyhow::Result;

/// Trait that abstracts the view rendering operations.
///
/// This allows the controller to work with either a real ViewManager
/// or a mock implementation during testing. The trait captures the three
/// levels of rendering optimization that the application uses, plus
/// terminal management methods.
pub trait ViewRenderer {
    /// Called when only cursor position needs updating (fastest)
    ///
    /// This is the most lightweight update, typically just repositioning
    /// the cursor without redrawing content.
    fn render_cursor_only(&mut self, state: &AppState) -> Result<()>;

    /// Called when content in a pane has changed (moderate cost)
    ///
    /// This updates the content of affected panes but doesn't redraw
    /// the entire interface.
    fn render_content_update(&mut self, state: &AppState) -> Result<()>;

    /// Called when full screen redraw is needed (most expensive)
    ///
    /// This performs a complete redraw of the entire interface,
    /// including all panes, status bar, and UI elements.
    fn render_full(&mut self, state: &AppState) -> Result<()>;

    /// Initialize the terminal for the application
    ///
    /// Sets up the terminal environment, alternate screen, etc.
    fn initialize_terminal(&self, state: &AppState) -> Result<()>;

    /// Clean up the terminal when the application exits
    ///
    /// Restores the terminal to its original state.
    fn cleanup_terminal(&self) -> Result<()>;
}
