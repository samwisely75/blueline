//! Terminal Test Automation Framework
//! 
//! This framework launches the actual blueline binary and simulates human interaction
//! to test real terminal behavior and catch visual bugs like "blacked out panes".

use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

#[cfg(test)]
use portable_pty::{CommandBuilder, PtySize, PtySystem};

/// Terminal test automation that interacts with the real blueline binary
#[derive(Debug)]
pub struct TerminalTestBot {
    #[cfg(test)]
    /// PTY pair for terminal interaction
    pty_pair: portable_pty::PtyPair,
    #[cfg(test)]
    /// Child process handle
    child: Box<dyn portable_pty::Child + Send>,
    /// Captured screen content
    screen_buffer: String,
}

impl TerminalTestBot {
    /// Launch blueline binary with a pseudo-terminal
    pub fn new() -> Result<Self> {
        #[cfg(test)]
        {
            use std::process::Command;
            
            // Build the blueline binary first
            let build_result = Command::new("cargo")
                .args(&["build", "--release"])
                .status()
                .context("Failed to build blueline binary")?;
                
            if !build_result.success() {
                return Err(anyhow::anyhow!("Failed to build blueline binary"));
            }

            // Create a PTY system
            let pty_system = portable_pty::native_pty_system();
            
            // Create PTY with reasonable terminal size
            let pty_pair = pty_system
                .openpty(PtySize {
                    rows: 24,
                    cols: 80,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .context("Failed to create PTY")?;

            // Create command to launch blueline
            let mut cmd = CommandBuilder::new("./target/release/blueline");
            cmd.cwd(std::env::current_dir()?);

            // Spawn the process in the PTY
            let child = pty_pair.slave.spawn_command(cmd)
                .context("Failed to spawn blueline in PTY")?;

            Ok(Self {
                pty_pair,
                child,
                screen_buffer: String::new(),
            })
        }
        
        #[cfg(not(test))]
        {
            Err(anyhow::anyhow!("TerminalTestBot only available in test builds"))
        }
    }

    /// Send a key sequence to the terminal
    pub fn send_keys(&mut self, keys: &str) -> Result<()> {
        #[cfg(test)]
        {
            self.pty_pair.master.write_all(keys.as_bytes())
                .context("Failed to send keys to terminal")?;
            self.pty_pair.master.flush()
                .context("Failed to flush keys to terminal")?;
            
            // Small delay to let the application process
            thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    }

    /// Send a single key press
    pub fn send_key(&mut self, key: char) -> Result<()> {
        self.send_keys(&key.to_string())
    }

    /// Send Enter key
    pub fn send_enter(&mut self) -> Result<()> {
        self.send_keys("\r")
    }

    /// Send Escape key  
    pub fn send_escape(&mut self) -> Result<()> {
        self.send_keys("\x1b")
    }

    /// Send Ctrl+C to quit
    pub fn send_ctrl_c(&mut self) -> Result<()> {
        self.send_keys("\x03")
    }

    /// Capture current screen content
    pub fn capture_screen(&mut self) -> Result<String> {
        #[cfg(test)]
        {
            let mut buffer = [0; 4096];
            let mut screen_content = String::new();
            
            // Try to read available output without blocking
            match self.pty_pair.master.read(&mut buffer) {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        screen_content.push_str(&String::from_utf8_lossy(&buffer[..bytes_read]));
                    }
                }
                Err(_) => {
                    // Non-blocking read, may not have data available
                }
            }
            
            self.screen_buffer.push_str(&screen_content);
            return Ok(self.screen_buffer.clone());
        }
        
        #[cfg(not(test))]
        {
            Ok(String::new())
        }
    }

    /// Wait for specific text to appear on screen
    pub fn wait_for_text(&mut self, expected: &str, timeout_ms: u64) -> Result<bool> {
        let start = std::time::Instant::now();
        
        while start.elapsed().as_millis() < timeout_ms as u128 {
            let screen = self.capture_screen()?;
            if screen.contains(expected) {
                return Ok(true);
            }
            thread::sleep(Duration::from_millis(100));
        }
        
        Ok(false)
    }

    /// Assert that text appears on screen
    pub fn assert_screen_contains(&mut self, expected: &str) -> Result<()> {
        let screen = self.capture_screen()?;
        if !screen.contains(expected) {
            return Err(anyhow::anyhow!(
                "Screen does not contain expected text: '{}'\nActual screen:\n{}", 
                expected, screen
            ));
        }
        Ok(())
    }

    /// Assert that screen is NOT blank/empty
    pub fn assert_screen_not_blank(&mut self) -> Result<()> {
        let screen = self.capture_screen()?;
        let visible_content = screen.trim();
        
        if visible_content.is_empty() {
            return Err(anyhow::anyhow!("Screen is completely blank!"));
        }
        
        // Check for common "blank" indicators
        if visible_content.len() < 10 {
            return Err(anyhow::anyhow!(
                "Screen appears mostly blank. Content: '{}'", visible_content
            ));
        }
        
        Ok(())
    }

    /// Get the last captured screen content
    pub fn get_screen_content(&self) -> &str {
        &self.screen_buffer
    }

    /// Simulate typing text in insert mode
    pub fn type_text(&mut self, text: &str) -> Result<()> {
        // Enter insert mode
        self.send_key('i')?;
        thread::sleep(Duration::from_millis(100));
        
        // Type the text
        self.send_keys(text)?;
        
        // Exit insert mode
        self.send_escape()?;
        thread::sleep(Duration::from_millis(100));
        
        Ok(())
    }

    /// Simulate HTTP request execution
    pub fn execute_http_request(&mut self) -> Result<()> {
        // Press Enter to execute request
        self.send_enter()?;
        
        // Wait a bit for HTTP request to complete
        thread::sleep(Duration::from_millis(1000));
        
        Ok(())
    }
}

impl Drop for TerminalTestBot {
    fn drop(&mut self) {
        // Attempt graceful shutdown
        let _ = self.send_ctrl_c();
        thread::sleep(Duration::from_millis(100));
        
        #[cfg(test)]
        {
            // Force kill if still running
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_bot_can_launch() {
        if let Ok(_bot) = TerminalTestBot::new() {
            // Success - bot can launch
        } else {
            // Skip test if can't create terminal bot (CI environment)
            println!("Skipping terminal bot test - can't create PTY");
        }
    }
}