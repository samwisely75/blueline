//! # Mode Commands Module
//!
//! This module contains commands that handle mode transitions and prefix modes
//! in the editor, such as entering and canceling various prefix modes.

pub mod append_after_cursor;
pub mod append_at_end_of_line;
pub mod cancel_g_prefix;
pub mod cancel_prefix_mode;
pub mod enter_command_mode;
pub mod enter_d_prefix;
pub mod enter_g_prefix;
pub mod enter_insert_mode;
pub mod enter_visual_block_append_mode;
pub mod enter_visual_block_change_mode;
pub mod enter_visual_block_insert_mode;
pub mod enter_visual_block_mode;
pub mod enter_visual_line_mode;
pub mod enter_visual_mode;
pub mod enter_y_prefix;
pub mod ex_command_mode;
pub mod exit_insert_mode;
pub mod exit_visual_block_insert_mode;
pub mod exit_visual_mode;
pub mod insert_at_beginning_of_line;

// Re-export all commands using glob imports to prevent merge conflicts
pub use append_after_cursor::AppendAfterCursorCommand;
pub use append_at_end_of_line::AppendAtEndOfLineCommand;
pub use cancel_g_prefix::CancelGPrefixCommand;
pub use cancel_prefix_mode::CancelPrefixModeCommand;
pub use enter_command_mode::EnterCommandModeCommand;
pub use enter_d_prefix::EnterDPrefixCommand;
pub use enter_g_prefix::EnterGPrefixCommand;
pub use enter_insert_mode::EnterInsertModeCommand;
pub use enter_visual_block_append_mode::EnterVisualBlockAppendModeCommand;
pub use enter_visual_block_change_mode::EnterVisualBlockChangeModeCommand;
pub use enter_visual_block_insert_mode::EnterVisualBlockInsertModeCommand;
pub use enter_visual_block_mode::EnterVisualBlockModeCommand;
pub use enter_visual_line_mode::EnterVisualLineModeCommand;
pub use enter_visual_mode::EnterVisualModeCommand;
pub use enter_y_prefix::EnterYPrefixCommand;
pub use ex_command_mode::*;
pub use exit_insert_mode::ExitInsertModeCommand;
pub use exit_visual_block_insert_mode::ExitVisualBlockInsertModeCommand;
pub use exit_visual_mode::ExitVisualModeCommand;
pub use insert_at_beginning_of_line::InsertAtBeginningOfLineCommand;
