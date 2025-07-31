//! Debug Capture Framework
//! 
//! Comprehensive debugging system to capture real application state
//! and identify rendering/input issues.

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use chrono::Local;
use crate::repl::events::Pane;
use crate::repl::view_models::ViewModel;
use crossterm::event::KeyEvent;

static DEBUG_LOG: Mutex<Option<std::fs::File>> = Mutex::new(None);

pub struct DebugCapture;

impl DebugCapture {
    /// Initialize debug capture system
    pub fn init() -> Result<(), std::io::Error> {
        let mut log = DEBUG_LOG.lock().unwrap();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("blueline_debug_capture.log")?;
        *log = Some(file);
        
        Self::log("=== DEBUG CAPTURE SESSION STARTED ===");
        Ok(())
    }
    
    /// Log a message with timestamp
    fn log(message: &str) {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let entry = format!("[{}] {}\n", timestamp, message);
        
        if let Ok(mut log) = DEBUG_LOG.lock() {
            if let Some(ref mut file) = *log {
                let _ = file.write_all(entry.as_bytes());
                let _ = file.flush();
            }
        }
        
        // Don't print to console to avoid blocking terminal apps
    }
    
    /// Capture full application state
    pub fn capture_full_state(view_model: &ViewModel, context: &str) {
        Self::log(&format!("=== FULL STATE CAPTURE: {} ===", context));
        
        // Basic state
        Self::log(&format!("Mode: {:?}", view_model.get_mode()));
        Self::log(&format!("Current Pane: {:?}", view_model.get_current_pane()));
        Self::log(&format!("Cursor Position: {:?}", view_model.get_cursor_position()));
        Self::log(&format!("Display Cursor: {:?}", view_model.get_display_cursor_position()));
        
        // Terminal size
        let (width, height) = view_model.terminal_size();
        Self::log(&format!("Terminal Size: {}x{}", width, height));
        
        // Content
        let request_text = view_model.get_request_text();
        let response_text = view_model.get_response_text();
        Self::log(&format!("Request Text: {:?}", request_text));
        Self::log(&format!("Request Text Length: {}", request_text.len()));
        Self::log(&format!("Response Text: {:?}", if response_text.len() > 100 { 
            format!("{}... [{}bytes]", &response_text[..100], response_text.len())
        } else { 
            response_text 
        }));
        Self::log(&format!("Response Status: {:?}", view_model.get_response_status_code()));
        
        // Display lines for both panes
        Self::capture_display_lines(view_model, Pane::Request, 10);
        Self::capture_display_lines(view_model, Pane::Response, 10);
        
        Self::log("=== END FULL STATE CAPTURE ===");
    }
    
    /// Capture display lines for a pane
    fn capture_display_lines(view_model: &ViewModel, pane: Pane, max_lines: usize) {
        Self::log(&format!("--- Display Lines for {:?} Pane ---", pane));
        
        let display_lines = view_model.get_display_lines_for_rendering(pane, 0, max_lines);
        
        if display_lines.is_empty() {
            Self::log("  ❌ NO DISPLAY LINES - COMPLETELY EMPTY!");
        } else {
            for (i, line_data) in display_lines.iter().enumerate() {
                match line_data {
                    Some((content, line_num, is_continuation, logical_start_col, logical_line)) => {
                        Self::log(&format!("  Line {}: {:?} (line_num={:?}, cont={}, logical={}:{}, len={})", 
                            i, content, line_num, is_continuation, logical_line, logical_start_col, content.len()));
                    }
                    None => {
                        Self::log(&format!("  Line {}: <BEYOND CONTENT>", i));
                    }
                }
            }
        }
        
        Self::log(&format!("--- End {:?} Pane Display Lines ---", pane));
    }
    
    /// Capture key event with full context
    pub fn capture_key_event(key_event: KeyEvent, view_model: &ViewModel, events_generated: &[crate::repl::commands::CommandEvent]) {
        Self::log(&format!("=== KEY EVENT: {:?} ===", key_event));
        Self::log(&format!("Context - Mode: {:?}, Pane: {:?}, Cursor: {:?}", 
            view_model.get_mode(), view_model.get_current_pane(), view_model.get_cursor_position()));
        
        if events_generated.is_empty() {
            Self::log("  ❌ NO COMMAND EVENTS GENERATED - KEY IGNORED!");
        } else {
            Self::log(&format!("  ✅ Generated {} events:", events_generated.len()));
            for (i, event) in events_generated.iter().enumerate() {
                Self::log(&format!("    {}: {:?}", i, event));
            }
        }
        
        Self::log("=== END KEY EVENT ===");
    }
    
    /// Capture rendering attempt
    pub fn capture_rendering_attempt(pane: Pane, visible_lines: usize, actual_content_lines: usize) {
        Self::log(&format!("=== RENDERING {:?} PANE ===", pane));
        Self::log(&format!("Visible Lines Requested: {}", visible_lines));
        Self::log(&format!("Actual Content Lines Available: {}", actual_content_lines));
        
        if actual_content_lines == 0 {
            Self::log("  ❌ NO CONTENT TO RENDER - PANE WILL BE BLANK!");
        } else if visible_lines == 0 {
            Self::log("  ❌ NO VISIBLE LINES - PANE HEIGHT TOO SMALL!");
        } else {
            Self::log("  ✅ Content available for rendering");
        }
        
        Self::log("=== END RENDERING ===");
    }
    
    /// Capture HTTP request/response flow
    pub fn capture_http_flow(stage: &str, details: &str) {
        Self::log(&format!("=== HTTP FLOW: {} ===", stage));
        Self::log(details);
        Self::log("=== END HTTP FLOW ===");
    }
    
    /// Capture display cache state
    pub fn capture_display_cache_state(pane: Pane, cache_size: usize, terminal_width: u16) {
        Self::log(&format!("=== DISPLAY CACHE {:?} ===", pane));
        Self::log(&format!("Cache Size: {} lines", cache_size));
        Self::log(&format!("Terminal Width: {}", terminal_width));
        
        let content_width = (terminal_width as usize).saturating_sub(4);
        Self::log(&format!("Calculated Content Width: {}", content_width));
        
        if cache_size == 0 {
            Self::log("  ❌ EMPTY DISPLAY CACHE - NO CONTENT TO RENDER!");
        } else {
            Self::log("  ✅ Display cache has content");
        }
        
        Self::log("=== END DISPLAY CACHE ===");
    }
    
    /// Test the debug framework
    pub fn test_framework() {
        Self::log("🧪 Debug framework test - if you see this, logging works!");
    }
}