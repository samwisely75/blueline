//! # UI Context
//!
//! Manages UI-specific state and operations including:
//! - Terminal dimensions and layout
//! - Profile information display
//! - UI coordination between different components
//!
//! This provides a clean separation of UI concerns from editor and HTTP domains.

/// UI context managing UI-specific state and operations
///
/// This context encapsulates all UI-related functionality that was previously
/// scattered throughout AppState, providing cleaner domain separation.
#[derive(Debug)]
pub struct UIContext {
    /// Current profile information for display
    profile_name: String,
    profile_path: String,
}

impl UIContext {
    /// Create a new UIContext with default values
    pub fn new() -> Self {
        Self {
            profile_name: String::new(),
            profile_path: String::new(),
        }
    }

    /// Set the profile information for display
    pub fn set_profile_info(&mut self, profile_name: String, profile_path: String) {
        self.profile_name = profile_name;
        self.profile_path = profile_path;
    }

    /// Get the current profile name
    pub fn profile_name(&self) -> &str {
        &self.profile_name
    }

    /// Get the current profile path
    pub fn profile_path(&self) -> &str {
        &self.profile_path
    }
}

impl Default for UIContext {
    fn default() -> Self {
        Self::new()
    }
}
