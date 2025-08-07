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

    tracing_subscriber::registry()
        .with(filter)
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
        .run_and_exit("tests/features") // Run all feature files in the tests/features directory
        .await;
}
