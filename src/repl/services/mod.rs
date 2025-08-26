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

pub mod http;
pub mod yank;

// Re-export service types
pub use http::{BufferRequestArgs, HttpExecutionResult, HttpResponseMessage, HttpService};
pub use yank::YankService;

/// Aggregates all services for convenient access
pub struct Services {
    /// Service for HTTP request operations (optional until configured)
    pub http: Option<HttpService>,
    /// Service for yank/paste operations
    pub yank: YankService,
}

impl Services {
    /// Create new Services without HTTP (needs to be configured with profile)
    pub fn new() -> Self {
        Self {
            http: None,
            yank: YankService::new(),
        }
    }

    /// Configure HTTP service with a profile
    pub fn configure_http(
        &mut self,
        profile: &impl bluenote::HttpConnectionProfile,
    ) -> anyhow::Result<()> {
        tracing::debug!("Configuring HTTP service with profile: {:?}", profile);
        match HttpService::new(profile) {
            Ok(service) => {
                tracing::info!("HTTP service configured successfully");
                self.http = Some(service);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to configure HTTP service: {}", e);
                Err(e)
            }
        }
    }
}

impl Default for Services {
    fn default() -> Self {
        Self::new()
    }
}
