use cucumber::World;

pub mod common;

pub use common::BluelineWorld;

/// Integration tests using Cucumber BDD framework
/// Run with: cargo test --test integration_tests
#[tokio::main]
async fn main() {
    BluelineWorld::run("features").await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_integration_tests() {
        BluelineWorld::run("features").await;
    }
}
