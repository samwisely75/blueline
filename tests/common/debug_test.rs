#[cfg(test)]
mod tests {

    use anyhow::Result;

    #[tokio::test]
    async fn test_basic_spawn() -> Result<()> {
        println!("Testing basic tokio::spawn...");

        let task = tokio::spawn(async {
            println!("Inside spawned task");
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            println!("Spawned task complete");
            42
        });

        println!("Waiting for spawned task...");
        let result = task.await.expect("Task failed");
        println!("Got result: {result}");

        Ok(())
    }

    #[tokio::test]
    async fn test_app_controller_creation() -> Result<()> {
        use blueline::cmd_args::CommandLineArgs;
        use blueline::repl::controllers::app_controller::AppController;
        use blueline::repl::io::test_bridge::{BridgedEventStream, BridgedRenderStream};

        println!("1. Testing bridge creation...");
        let (event_stream, _controller) = BridgedEventStream::new();
        let (render_stream, _monitor) = BridgedRenderStream::new((80, 24));
        println!("✅ Bridge created");

        println!("2. Testing command args...");
        let cmd_args = CommandLineArgs::parse_from(vec!["blueline".to_string()]);
        println!("✅ Command args parsed");

        println!("3. Testing AppController creation...");
        let app_result = AppController::with_io_streams(cmd_args, event_stream, render_stream);
        match app_result {
            Ok(_app) => {
                println!("✅ AppController created successfully!");
                println!("4. Skipping app.run() test to avoid hanging - creation test complete!");

                // For now, just test that we can create the AppController successfully
                // The app.run() method blocks indefinitely waiting for events, which is expected behavior
                // In real usage, it would be terminated by Ctrl+C or other quit signals
            }
            Err(e) => {
                println!("❌ AppController creation failed: {e}");
                return Err(e);
            }
        }

        println!("SUCCESS: AppController creation works!");
        Ok(())
    }
}
