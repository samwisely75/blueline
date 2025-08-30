//! # Unified Command Pattern Infrastructure  
//!
//! This module implements the new unified Command Pattern architecture that
//! replaces the old command system. Commands contain both key binding logic
//! (is_relevant) and business logic (handle), emitting semantic ModelEvents.

// Core infrastructure
pub mod command;
pub mod dynamic_registry; // Dynamic command discovery system
pub mod events;

// Command implementations (migrated)
pub mod change_selection;
pub mod cut_character;
pub mod cut_current_line;
pub mod cut_selection;
pub mod cut_to_end_of_line;
pub mod delete_selection;
pub mod exit_visual_block_insert;
pub mod http;
pub mod paste; // Added missing paste module
pub mod repeat_visual_selection;
pub mod setting_change;
pub mod show_profile;
pub mod visual_block_append;
pub mod visual_block_insert;
pub mod yank;
pub mod yank_current_line;

// Placeholder reservations for remaining migrations (issues #231-295)
// Agents: Replace your assigned comment line with: pub mod your_command_name;
// Issue #231: MoveLeftCommand
// pub mod move_left;
// Issue #232: MoveRightCommand
pub mod move_right;
// Issue #233: MoveUpCommand
// pub mod move_up;
// Issue #234: MoveDownCommand
// pub mod move_down;
// Issue #235: InsertCharCommand
// pub mod insert_char;
// Issue #236: DeleteCharCommand
// pub mod delete_char;
// Issue #237: DeleteCharAtCursorCommand
// pub mod delete_char_at_cursor;
// Issue #238: InsertTabCommand
// pub mod insert_tab;
// Issue #239: InsertNewlineCommand
// pub mod insert_newline;
// Issue #240: BeginningOfLineCommand
// pub mod beginning_of_line;
// Issue #241: EndOfLineCommand
// pub mod end_of_line;
// Issue #242: NextWordCommand
// pub mod next_word;
// Issue #243: PreviousWordCommand
// pub mod previous_word;
// Issue #244: EndOfWordCommand
// pub mod end_of_word;
// Issue #245: PageUpCommand
// pub mod page_up;
// Issue #246: PageDownCommand
// pub mod page_down;
// Issue #247: GoToTopCommand
// pub mod go_to_top;
// Issue #248: GoToBottomCommand
// pub mod go_to_bottom;
// Issue #249: EnterVisualModeCommand
// pub mod enter_visual_mode;
// Issue #250: ExitVisualModeCommand
// pub mod exit_visual_mode;
// Issue #251: EnterCommandModeCommand
// pub mod enter_command_mode;
// Issue #252: ExitCommandModeCommand
// pub mod exit_command_mode;
// Issue #253: EnterInsertModeCommand
// pub mod enter_insert_mode;
// Issue #254: ExitInsertModeCommand
// pub mod exit_insert_mode;
// Issue #255: AppendAfterCursorCommand
// pub mod append_after_cursor;
// Issue #256: AppendAtEndOfLineCommand
// pub mod append_at_end_of_line;
// Issue #257: InsertAtBeginningOfLineCommand
// pub mod insert_at_beginning_of_line;
// Issue #258: SwitchPaneCommand
// pub mod switch_pane;
// Issue #259: ExecuteRequestCommand
// pub mod execute_request;
// Issue #260: EnterDPrefixCommand
// pub mod enter_d_prefix;
// Issue #261: EnterYPrefixCommand
// pub mod enter_y_prefix;
// Issue #262: EnterGModeCommand
// pub mod enter_g_mode;
// Issue #263: EnterGPrefixCommand
// pub mod enter_g_prefix;
// Issue #264: ExCommand
// pub mod ex_command;
// Issue #265: AppTerminateCommand
// pub mod app_terminate;
// Issue #266: HomeKeyCommand
// pub mod home_key;
// Issue #267: EndKeyCommand
// pub mod end_key;
// Issue #268: HalfPageUpCommand
// pub mod half_page_up;
// Issue #269: HalfPageDownCommand
// pub mod half_page_down;
// Issue #270: Multi-cursor text commands (reserved)
// pub mod multi_cursor_insert;
// pub mod multi_cursor_delete;
// Issue #275: Additional navigation commands
// pub mod move_cursor_left_arrow;
// pub mod move_cursor_right_arrow;
// pub mod move_cursor_up_arrow;
// pub mod move_cursor_down_arrow;
// Issue #280: Additional visual mode commands
// pub mod visual_line_mode;
// pub mod visual_block_mode;
// Issue #285: Additional editing commands
// pub mod insert_line_above;
// pub mod insert_line_below;
// pub mod delete_line;
// pub mod duplicate_line;
// Issue #290: Additional utility commands
// pub mod undo;
// pub mod redo;
// pub mod save;
// pub mod quit;
// Issue #295: Final batch commands
// pub mod search;
// pub mod replace;
// pub mod goto_line;

// Re-export main types
pub use command::{Command, CommandContext, ExecutionContext};
pub use dynamic_registry::{
    register_all_commands, CommandEntry, CommandFactory, DynamicCommandRegistry,
};
pub use events::{ModelEvent, YankType};

// Note: Individual command re-exports no longer needed -
// commands self-register using the inventory system
