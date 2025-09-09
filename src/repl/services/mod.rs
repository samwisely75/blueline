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

pub mod async_word_segmenter;
pub mod http;
pub mod word_segmenter;
pub mod yank;

// Re-export service types
pub use async_word_segmenter::{
    AsyncWordSegmenter, SegmentationResult as AsyncSegmentationResult, SegmentationTask,
};
pub use http::{BufferRequestArgs, HttpExecutionResult, HttpResponseMessage, HttpService};
pub use word_segmenter::{
    SegmentationResult, WordBoundaries, WordFlags, WordSegmenter, WordSegmenterService,
};
pub use yank::YankService;

/// Aggregates all services for convenient access
pub struct Services {
    /// Service for HTTP request operations (optional until configured)
    pub http: Option<HttpService>,
    /// Service for word segmentation and boundary detection
    pub word_segmenter: std::sync::Arc<WordSegmenterService>,
    /// Async word segmentation service for non-blocking operations
    pub async_word_segmenter: AsyncWordSegmenter,
    /// Service for yank/paste operations
    pub yank: YankService,
}

impl Services {
    /// Create new Services without HTTP (needs to be configured with profile)
    pub fn new() -> Self {
        Self {
            http: None,
            word_segmenter: WordSegmenterService::new(),
            async_word_segmenter: AsyncWordSegmenter::new(),
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
