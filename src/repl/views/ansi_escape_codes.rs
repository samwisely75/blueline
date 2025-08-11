//! ANSI escape code constants for terminal styling
//!
//! This module provides a comprehensive set of ANSI escape codes for terminal formatting,
//! including colors, text attributes, and cursor controls.

#![allow(dead_code)]

// ============================================================================
// TEXT ATTRIBUTES
// ============================================================================

pub const RESET: &str = "\x1b[0m"; // Reset all attributes
pub const BOLD: &str = "\x1b[1m"; // Bold text
pub const DIM: &str = "\x1b[2m"; // Dimmed/faint text
pub const ITALIC: &str = "\x1b[3m"; // Italic text (not widely supported)
pub const UNDERLINE: &str = "\x1b[4m"; // Underlined text
pub const BLINK: &str = "\x1b[5m"; // Blinking text (not widely supported)
pub const REVERSE: &str = "\x1b[7m"; // Reverse video (swap fg/bg)
pub const HIDDEN: &str = "\x1b[8m"; // Hidden text
pub const STRIKETHROUGH: &str = "\x1b[9m"; // Strikethrough text

// ============================================================================
// STANDARD FOREGROUND COLORS (30-37)
// ============================================================================

pub const FG_BLACK: &str = "\x1b[30m";
pub const FG_RED: &str = "\x1b[31m";
pub const FG_GREEN: &str = "\x1b[32m";
pub const FG_YELLOW: &str = "\x1b[33m";
pub const FG_BLUE: &str = "\x1b[34m";
pub const FG_MAGENTA: &str = "\x1b[35m";
pub const FG_CYAN: &str = "\x1b[36m";
pub const FG_WHITE: &str = "\x1b[37m";

// ============================================================================
// STANDARD BACKGROUND COLORS (40-47)
// ============================================================================

pub const BG_BLACK: &str = "\x1b[40m";
pub const BG_RED: &str = "\x1b[41m";
pub const BG_GREEN: &str = "\x1b[42m";
pub const BG_YELLOW: &str = "\x1b[43m";
pub const BG_BLUE: &str = "\x1b[44m";
pub const BG_MAGENTA: &str = "\x1b[45m";
pub const BG_CYAN: &str = "\x1b[46m";
pub const BG_WHITE: &str = "\x1b[47m";

// ============================================================================
// BRIGHT/HIGH INTENSITY FOREGROUND COLORS (90-97)
// ============================================================================

pub const FG_BRIGHT_BLACK: &str = "\x1b[90m"; // Also known as dark gray
pub const FG_BRIGHT_RED: &str = "\x1b[91m";
pub const FG_BRIGHT_GREEN: &str = "\x1b[92m";
pub const FG_BRIGHT_YELLOW: &str = "\x1b[93m";
pub const FG_BRIGHT_BLUE: &str = "\x1b[94m";
pub const FG_BRIGHT_MAGENTA: &str = "\x1b[95m";
pub const FG_BRIGHT_CYAN: &str = "\x1b[96m";
pub const FG_BRIGHT_WHITE: &str = "\x1b[97m";

// ============================================================================
// BRIGHT/HIGH INTENSITY BACKGROUND COLORS (100-107)
// ============================================================================

pub const BG_BRIGHT_BLACK: &str = "\x1b[100m"; // Also known as dark gray
pub const BG_BRIGHT_RED: &str = "\x1b[101m";
pub const BG_BRIGHT_GREEN: &str = "\x1b[102m";
pub const BG_BRIGHT_YELLOW: &str = "\x1b[103m";
pub const BG_BRIGHT_BLUE: &str = "\x1b[104m";
pub const BG_BRIGHT_MAGENTA: &str = "\x1b[105m";
pub const BG_BRIGHT_CYAN: &str = "\x1b[106m";
pub const BG_BRIGHT_WHITE: &str = "\x1b[107m";

// ============================================================================
// 256 COLOR MODE - FOREGROUND COLORS (38;5;n)
// ============================================================================

// Blue shades (matching the background ones for consistency)
pub const FG_256_VERY_DARK_BLUE: &str = "\x1b[38;5;17m";
pub const FG_256_DARK_BLUE: &str = "\x1b[38;5;18m";
pub const FG_256_DARK_BLUE_2: &str = "\x1b[38;5;19m";
pub const FG_256_DARK_BLUE_3: &str = "\x1b[38;5;20m";
pub const FG_256_BLUE: &str = "\x1b[38;5;21m";
pub const FG_256_DEEP_SKY_BLUE: &str = "\x1b[38;5;25m";
pub const FG_256_DODGER_BLUE: &str = "\x1b[38;5;26m";
pub const FG_256_DODGER_BLUE_2: &str = "\x1b[38;5;27m";
pub const FG_256_STEEL_BLUE: &str = "\x1b[38;5;32m";
pub const FG_256_LIGHT_STEEL_BLUE: &str = "\x1b[38;5;33m";
pub const FG_256_DEEP_SKY_BLUE_2: &str = "\x1b[38;5;39m";
pub const FG_256_TURQUOISE: &str = "\x1b[38;5;45m";
pub const FG_256_CYAN_LIGHT: &str = "\x1b[38;5;51m";
pub const FG_256_SLATE_BLUE: &str = "\x1b[38;5;61m";
pub const FG_256_SLATE_BLUE_2: &str = "\x1b[38;5;62m";
pub const FG_256_ROYAL_BLUE: &str = "\x1b[38;5;63m";
pub const FG_256_STEEL_BLUE_DARK: &str = "\x1b[38;5;68m";
pub const FG_256_CORNFLOWER_BLUE: &str = "\x1b[38;5;69m";
pub const FG_256_STEEL_BLUE_LIGHT: &str = "\x1b[38;5;75m";

// Additional useful foreground colors
pub const FG_256_ORANGE: &str = "\x1b[38;5;208m";
pub const FG_256_ORANGE_BRIGHT: &str = "\x1b[38;5;214m";
pub const FG_256_PURPLE: &str = "\x1b[38;5;135m";
pub const FG_256_PURPLE_BRIGHT: &str = "\x1b[38;5;141m";
pub const FG_256_PINK: &str = "\x1b[38;5;205m";
pub const FG_256_PINK_HOT: &str = "\x1b[38;5;197m";
pub const FG_256_GRAY_DARK: &str = "\x1b[38;5;240m";
pub const FG_256_GRAY: &str = "\x1b[38;5;244m";
pub const FG_256_GRAY_LIGHT: &str = "\x1b[38;5;250m";
pub const FG_256_GOLD: &str = "\x1b[38;5;220m";
pub const FG_256_LIME: &str = "\x1b[38;5;154m";
pub const FG_256_AQUA: &str = "\x1b[38;5;86m";

// ============================================================================
// 256 COLOR MODE - BACKGROUND COLORS (48;5;n)
// ============================================================================

// Blue shades (for selection highlighting)
pub const BG_256_VERY_DARK_BLUE: &str = "\x1b[48;5;17m";
pub const BG_256_DARK_BLUE: &str = "\x1b[48;5;18m";
pub const BG_256_DARK_BLUE_2: &str = "\x1b[48;5;19m";
pub const BG_256_DARK_BLUE_3: &str = "\x1b[48;5;20m";
pub const BG_256_BLUE: &str = "\x1b[48;5;21m";
pub const BG_256_DEEP_SKY_BLUE: &str = "\x1b[48;5;25m";
pub const BG_256_DODGER_BLUE: &str = "\x1b[48;5;26m";
pub const BG_256_DODGER_BLUE_2: &str = "\x1b[48;5;27m";
pub const BG_256_STEEL_BLUE: &str = "\x1b[48;5;32m";
pub const BG_256_LIGHT_STEEL_BLUE: &str = "\x1b[48;5;33m";
pub const BG_256_DEEP_SKY_BLUE_2: &str = "\x1b[48;5;39m";
pub const BG_256_TURQUOISE: &str = "\x1b[48;5;45m";
pub const BG_256_CYAN_LIGHT: &str = "\x1b[48;5;51m";
pub const BG_256_SLATE_BLUE: &str = "\x1b[48;5;61m";
pub const BG_256_SLATE_BLUE_2: &str = "\x1b[48;5;62m";
pub const BG_256_ROYAL_BLUE: &str = "\x1b[48;5;63m";
pub const BG_256_STEEL_BLUE_DARK: &str = "\x1b[48;5;68m";
pub const BG_256_CORNFLOWER_BLUE: &str = "\x1b[48;5;69m";
pub const BG_256_STEEL_BLUE_LIGHT: &str = "\x1b[48;5;75m";

// ============================================================================
// CURSOR CONTROL
// ============================================================================

pub const CURSOR_BLOCK: &str = "\x1b[2 q"; // Blinking block cursor
pub const CURSOR_BLOCK_STEADY: &str = "\x1b[1 q"; // Steady block cursor
pub const CURSOR_UNDERLINE: &str = "\x1b[4 q"; // Blinking underline cursor
pub const CURSOR_UNDERLINE_STEADY: &str = "\x1b[3 q"; // Steady underline cursor
pub const CURSOR_BAR: &str = "\x1b[6 q"; // Blinking bar cursor (I-beam)
pub const CURSOR_BAR_STEADY: &str = "\x1b[5 q"; // Steady bar cursor (I-beam)

// ============================================================================
// LINE CONTROL
// ============================================================================

pub const CLEAR_LINE: &str = "\x1b[K"; // Clear from cursor to end of line
pub const CLEAR_LINE_ENTIRE: &str = "\x1b[2K"; // Clear entire line
pub const CLEAR_LINE_START: &str = "\x1b[1K"; // Clear from start of line to cursor

// ============================================================================
// SEMANTIC COLOR ALIASES (Application-specific meanings)
// ============================================================================

// Foreground semantic colors
pub const FG_NORMAL: &str = ""; // Normal text (no color change)
pub const FG_SELECTED: &str = FG_BRIGHT_WHITE; // Selected text
pub const FG_DIM_TEXT: &str = FG_BRIGHT_BLACK; // Dimmed text (dark grey) for empty lines
pub const FG_SUCCESS: &str = FG_GREEN; // Success indicator
pub const FG_ERROR: &str = FG_RED; // Error indicator
pub const FG_WARNING: &str = FG_YELLOW; // Warning/executing indicator
pub const FG_INFO: &str = FG_BLUE; // Info text

pub const FG_SEPARATOR: &str = FG_256_DEEP_SKY_BLUE; // Pane separator/boundary (blueline)

// Background semantic colors
pub const BG_NORMAL: &str = ""; // Normal background (no color change)
pub const BG_SELECTED: &str = BG_256_DEEP_SKY_BLUE; // Selected background (customize this!)
                                                    // Alternative selection backgrounds you can try:
                                                    // pub const BG_SELECTED: &str = BG_BLUE;        // Standard blue
                                                    // pub const BG_SELECTED: &str = BG_BRIGHT_BLUE; // Bright blue
                                                    // pub const BG_SELECTED: &str = BG_256_DARK_BLUE_2; // 256-color dark blue
                                                    // pub const BG_SELECTED: &str = BG_256_STEEL_BLUE; // 256-color steel blue

// ============================================================================
// COMPOUND STYLES (Pre-combined for convenience)
// ============================================================================

// Status indicators with bullets
pub const STATUS_BULLET_GREEN: &str = "\x1b[32m●\x1b[0m "; // Green bullet for success
pub const STATUS_BULLET_RED: &str = "\x1b[31m●\x1b[0m "; // Red bullet for errors
pub const STATUS_BULLET_YELLOW: &str = "\x1b[33m●\x1b[0m"; // Yellow bullet for executing
pub const STATUS_BULLET_DEFAULT: &str = "● "; // Default bullet

// ============================================================================
// HELPER FUNCTIONS (Optional utility functions)
// ============================================================================

/// Create a 256-color foreground escape code
pub fn fg_256(color: u8) -> String {
    format!("\x1b[38;5;{color}m")
}

/// Create a 256-color background escape code
pub fn bg_256(color: u8) -> String {
    format!("\x1b[48;5;{color}m")
}

/// Create an RGB foreground escape code (24-bit color)
pub fn fg_rgb(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{r};{g};{b}m")
}

/// Create an RGB background escape code (24-bit color)
pub fn bg_rgb(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[48;2;{r};{g};{b}m")
}
