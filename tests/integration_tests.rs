//! Integration tests for Blueline REPL
//!
//! This module sets up the Cucumber test runner for BDD-style integration testing.
//! Tests are defined in .feature files and implemented via step definitions.

use cucumber::World;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod common;
mod steps;

// Re-export the world for easy access
use common::world::BluelineWorld;

#[tokio::test]
async fn cucumber_integration_tests() {
    // Initialize tracing subscriber for tests
    // Use RUST_LOG environment variable to control log level
    // Example: RUST_LOG=debug cargo test --test integration_tests
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default to info level for blueline, warn for others
        EnvFilter::new("warn,blueline=info,integration_tests=info")
    });

    let registry = tracing_subscriber::registry().with(filter);

    // Check if we should log to file
    if let Some(log_file) = std::env::var_os("BLUELINE_LOG_FILE").and_then(|s| s.into_string().ok())
    {
        // Log to both console and file
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&log_file)
            .expect("Failed to create log file");

        registry
            .with(
                tracing_subscriber::fmt::layer()
                    .with_test_writer() // Console output
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .with_target(true)
                    .with_level(true)
                    .compact(),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(file) // File output
                    .with_thread_ids(true)
                    .with_thread_names(true)
                    .with_target(true)
                    .with_level(true)
                    .with_ansi(false), // No ANSI colors in file
            )
            .init();
    } else {
        // Console output only
        registry
            .with(
                tracing_subscriber::fmt::layer()
                    .with_test_writer() // Use test-friendly output
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .with_target(true)
                    .with_level(true)
                    .compact(), // Use compact formatting for tests
            )
            .init();
    }

    tracing::info!("Starting Blueline integration tests");

    // Configure and run Cucumber tests
    BluelineWorld::cucumber()
        .max_concurrent_scenarios(1) // Run scenarios sequentially to avoid state conflicts
        .before(|_feature, _rule, scenario, world| {
            Box::pin(async move {
                tracing::debug!("Starting scenario: {}", scenario.name);
                // Initialize world before each scenario
                world.initialize().await;
            })
        })
        .after(|_feature, _rule, scenario, _event, world| {
            Box::pin(async move {
                tracing::debug!("Finishing scenario: {}", scenario.name);
                // Clean up after each scenario
                if let Some(world) = world {
                    world.cleanup().await;
                }
            })
        })
        // Use standard output for test results
        .run(
            std::env::var_os("CUCUMBER_FEATURES")
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_else(|| "tests/features".to_string()),
        ) // Allow feature path override
        .await;
}
