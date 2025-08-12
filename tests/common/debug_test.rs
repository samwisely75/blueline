#[cfg(test)]
mod tests {

    use anyhow::Result;

    #[tokio::test]
    async fn test_basic_spawn() -> Result<()> {
        tracing::debug!("Testing basic tokio::spawn...");

        let task = tokio::spawn(async {
            tracing::debug!("Inside spawned task");
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            tracing::debug!("Spawned task complete");
            42
        });

        tracing::debug!("Waiting for spawned task...");
        let result = task.await.expect("Task failed");
        tracing::debug!("Got result: {result}");

        Ok(())
    }

    #[tokio::test]
    async fn test_app_controller_creation() -> Result<()> {
        use blueline::cmd_args::CommandLineArgs;
        use blueline::repl::controllers::app_controller::AppController;
        use blueline::repl::io::test_bridge::{BridgedEventStream, BridgedRenderStream};

        tracing::info!("1. Testing bridge creation...");
        let (event_stream, _controller) = BridgedEventStream::new();
        let (render_stream, _monitor) = BridgedRenderStream::new((80, 24));
        tracing::info!("✅ Bridge created");

        tracing::info!("2. Testing command args...");
        let cmd_args = CommandLineArgs::parse_from(vec!["blueline".to_string()]);
        tracing::info!("✅ Command args parsed");

        tracing::info!("3. Testing AppController creation...");
        let app_result = AppController::with_io_streams(cmd_args, event_stream, render_stream);
        match app_result {
            Ok(_app) => {
                tracing::info!("✅ AppController created successfully!");
                tracing::info!("4. Skipping app.run() test to avoid hanging - creation test complete!");

                // For now, just test that we can create the AppController successfully
                // The app.run() method blocks indefinitely waiting for events, which is expected behavior
                // In real usage, it would be terminated by Ctrl+C or other quit signals
            }
            Err(e) => {
                tracing::error!("❌ AppController creation failed: {e}");
                return Err(e);
            }
        }

        tracing::info!("SUCCESS: AppController creation works!");
        Ok(())
    }
}