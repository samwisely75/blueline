//! # BlueLine Main Entry Point
//!
//! Clean MVVM HTTP client with vim-style interface.

use anyhow::Result;
use blueline::AppController;

#[tokio::main]
async fn main() -> Result<()> {
    // Create and run the application controller
    let mut app = AppController::new()?;

    // Print welcome message before starting
    println!("🔵 BlueLine HTTP Client");
    println!("Press 'i' to enter insert mode, 'Esc' to exit insert mode");
    println!("Use 'h', 'j', 'k', 'l' or arrow keys to move cursor");
    println!("Press 'Tab' to switch between panes");
    println!("Press 'Enter' in normal mode to execute HTTP request");
    println!("Press 'Ctrl+C' to quit");
    println!("Starting application...\n");

    // Small delay to let user read the instructions
    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

    // Run the application
    app.run().await?;

    println!("\n👋 Thanks for using BlueLine!");
    Ok(())
}
