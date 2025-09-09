//! Asynchronous word segmentation service
//!
//! This service provides non-blocking word boundary detection by performing
//! segmentation on a background thread. This prevents UI freezing when processing
//! large files with very long lines.

use crate::repl::services::word_segmenter::{WordFlags, WordSegmenterService};
use std::sync::{
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};

/// Type alias for the complex receiver type to satisfy clippy
type ResultReceiver = Arc<Mutex<Receiver<SegmentationResult>>>;

/// Task to be processed by the background segmenter thread
#[derive(Debug)]
pub struct SegmentationTask {
    /// Unique identifier for the line being segmented
    pub line_id: usize,
    /// Text content to segment
    pub text: String,
    /// Generation ID to validate freshness of results
    pub generation_id: u64,
}

/// Result from background segmentation
#[derive(Debug)]
pub struct SegmentationResult {
    /// Line ID this result applies to
    pub line_id: usize,
    /// Word boundary flags for each character
    pub flags: Vec<WordFlags>,
    /// Generation ID to validate freshness
    pub generation_id: u64,
}

/// Asynchronous word segmenter using a single background thread
pub struct AsyncWordSegmenter {
    /// Channel for sending segmentation tasks to background thread
    task_sender: Sender<SegmentationTask>,
    /// Channel for receiving completed segmentation results
    result_receiver: ResultReceiver,
    /// Handle to background worker thread
    _worker_handle: JoinHandle<()>,
}

impl Default for AsyncWordSegmenter {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncWordSegmenter {
    /// Create a new async word segmenter with background thread
    pub fn new() -> Self {
        // Create channels for task and result communication
        let (task_sender, task_receiver) = mpsc::channel::<SegmentationTask>();
        let (result_sender, result_receiver) = mpsc::channel::<SegmentationResult>();

        // Wrap result receiver in Arc<Mutex> for thread-safe access
        let result_receiver = Arc::new(Mutex::new(result_receiver));

        // Spawn background worker thread
        let worker_handle = thread::spawn(move || {
            Self::worker_thread(task_receiver, result_sender);
        });

        Self {
            task_sender,
            result_receiver,
            _worker_handle: worker_handle,
        }
    }

    /// Request segmentation for a line of text
    /// This method is non-blocking and returns immediately
    pub fn request_segmentation(
        &self,
        line_id: usize,
        text: String,
        generation_id: u64,
    ) -> Result<(), String> {
        let task = SegmentationTask {
            line_id,
            text,
            generation_id,
        };

        self.task_sender.send(task).map_err(|_| {
            "Failed to send segmentation task - worker thread may have died".to_string()
        })
    }

    /// Try to receive completed segmentation results (non-blocking)
    /// Returns None if no results are available
    pub fn try_recv_results(&self) -> Vec<SegmentationResult> {
        let mut results = Vec::new();

        if let Ok(receiver) = self.result_receiver.try_lock() {
            // Drain all available results
            while let Ok(result) = receiver.try_recv() {
                results.push(result);
            }
        }

        results
    }

    /// Background worker thread that processes segmentation tasks
    fn worker_thread(
        task_receiver: Receiver<SegmentationTask>,
        result_sender: Sender<SegmentationResult>,
    ) {
        tracing::info!("AsyncWordSegmenter worker thread started");

        // Create the actual segmenter service
        let segmenter = WordSegmenterService::new();

        // Process tasks until channel is closed
        while let Ok(task) = task_receiver.recv() {
            tracing::debug!(
                "Processing segmentation task for line {} (gen {}, {} chars)",
                task.line_id,
                task.generation_id,
                task.text.len()
            );

            // Perform the actual segmentation
            let result = match segmenter.find_word_boundaries(&task.text) {
                Ok(boundaries) => {
                    let flags = boundaries.to_word_flags(&task.text);
                    SegmentationResult {
                        line_id: task.line_id,
                        flags,
                        generation_id: task.generation_id,
                    }
                }
                Err(e) => {
                    tracing::warn!("Segmentation failed for line {}: {}", task.line_id, e);
                    // Return empty flags on error
                    SegmentationResult {
                        line_id: task.line_id,
                        flags: vec![WordFlags::default(); task.text.chars().count()],
                        generation_id: task.generation_id,
                    }
                }
            };

            // Send result back to main thread
            if let Err(e) = result_sender.send(result) {
                tracing::error!("Failed to send segmentation result: {}", e);
                break;
            }
        }

        tracing::info!("AsyncWordSegmenter worker thread exiting");
    }
}

impl Drop for AsyncWordSegmenter {
    fn drop(&mut self) {
        tracing::debug!(
            "AsyncWordSegmenter dropping, worker thread will exit when task queue empties"
        );
        // When task_sender is dropped, worker thread will exit cleanly
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn async_segmenter_should_create() {
        let segmenter = AsyncWordSegmenter::new();
        // Just verify it constructs without panicking
        assert!(segmenter
            .task_sender
            .send(SegmentationTask {
                line_id: 0,
                text: "test".to_string(),
                generation_id: 1,
            })
            .is_ok());
    }

    #[test]
    fn async_segmenter_should_process_simple_task() {
        let segmenter = AsyncWordSegmenter::new();

        // Send a segmentation request
        segmenter
            .request_segmentation(0, "hello world".to_string(), 1)
            .expect("Should send task successfully");

        // Give worker thread time to process
        std::thread::sleep(Duration::from_millis(100));

        // Check for results
        let results = segmenter.try_recv_results();
        assert!(
            !results.is_empty(),
            "Should have received segmentation result"
        );

        let result = &results[0];
        assert_eq!(result.line_id, 0);
        assert_eq!(result.generation_id, 1);
        assert_eq!(result.flags.len(), 11); // "hello world" has 11 characters
    }

    #[test]
    fn async_segmenter_should_handle_empty_text() {
        let segmenter = AsyncWordSegmenter::new();

        segmenter
            .request_segmentation(0, "".to_string(), 1)
            .expect("Should handle empty text");

        std::thread::sleep(Duration::from_millis(50));

        let results = segmenter.try_recv_results();
        assert!(!results.is_empty());

        let result = &results[0];
        assert_eq!(result.flags.len(), 0);
    }
}
