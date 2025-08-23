//! # Services Layer
//!
//! Provides business logic services that are used by Commands.
//! Services encapsulate reusable operations and reduce coupling
//! between Commands and the ViewModel.
//!
//! Services should only exist when they add real value by:
//! - Managing their own state (like YankService with yank buffer)
//! - Providing complex business logic
//! - Abstracting external resources

pub mod yank;

// Re-export service types
pub use yank::YankService;

/// Aggregates all services for convenient access
pub struct Services {
    /// Service for yank/paste operations
    pub yank: YankService,
}

impl Services {
    /// Create new Services with default configurations
    pub fn new() -> Self {
        Self {
            yank: YankService::new(),
        }
    }
}

impl Default for Services {
    fn default() -> Self {
        Self::new()
    }
}
