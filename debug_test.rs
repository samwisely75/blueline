// Temporary debug script to isolate the hanging issue
use std::time::Duration;
use tokio;
use blueline::{
    cmd_args::CommandLineArgs,
    repl::{
        controllers::app_controller::AppController,
        io::test_bridge::{BridgedEventStream, BridgedRenderStream},
    },
};

#[tokio::main]
async fn main() {
    println!("Testing AppController creation...");
    
    let args = CommandLineArgs::parse_from(vec!["blueline".to_string()]);
    let (event_stream, _controller) = BridgedEventStream::new();
    let (render_stream, _monitor) = BridgedRenderStream::new((80, 24));
    
    println!("Creating AppController...");
    let app = AppController::with_io_streams(args, event_stream, render_stream);
    
    match app {
        Ok(_) => println!("✅ AppController created successfully"),
        Err(e) => println!("❌ AppController creation failed: {}", e),
    }
    
    println!("Done!");
}