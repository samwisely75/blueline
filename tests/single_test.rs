use cucumber::World;

pub mod common;

pub use common::world::BluelineWorld;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_single_test() {
        BluelineWorld::run("features/screen_refresh_single.feature").await;
    }
}
