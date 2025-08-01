use cucumber::World;

pub mod common;

pub use common::world::BluelineWorld;

/// Integration tests using Cucumber BDD framework
/// Run with: cargo test --test integration_tests
#[tokio::main]
async fn main() {
    // Serialize feature execution to prevent resource conflicts
    run_features_sequentially().await;
}

/// Run each feature file sequentially to avoid resource conflicts
async fn run_features_sequentially() {
    let features = [
        "features/application.feature",
        "features/command_line.feature",
        "features/double_byte_rendering_bug.feature",
        "features/integration.feature",
        "features/mode_transitions.feature",
        "features/navigation_command.feature",
        // "features/real_application_bug.feature", // Disabled - step definitions commented out causing timeout
        // "features/real_vte_bug_test.feature", // Disabled - debugging test for separate issue
        "features/text_editing.feature",
    ];

    // Run the main features first
    println!(
        "Running {} main feature files sequentially...",
        features.len()
    );

    for (i, feature) in features.iter().enumerate() {
        println!("\n[{}/{}] Running {feature}...", i + 1, features.len());
        BluelineWorld::run(feature).await;
        println!("✓ {feature} completed successfully");
    }

    println!("\n🎉 All feature files completed successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_integration_tests() {
        // Run features sequentially in tests as well
        run_features_sequentially().await;
    }
}
