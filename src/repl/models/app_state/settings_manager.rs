//! # Settings Management
//!
//! Handles settings changes from ex commands.

use super::AppState;
use crate::repl::models::settings::{Setting, SettingValue};
use anyhow::Result;

impl AppState {
    /// Apply a setting change from an ex command
    pub fn apply_setting(&mut self, setting: Setting, value: SettingValue) -> Result<()> {
        match setting {
            Setting::Wrap => {
                let enable = value == SettingValue::On;
                self.pane_manager.set_wrap_enabled(enable);
                // self.pane_manager.rebuild_display_caches_and_sync();
                // extend(visibility_events);
                Ok(())
            }
            Setting::LineNumbers => {
                let enable = value == SettingValue::On;
                self.pane_manager.set_line_numbers_visible(enable);
                // self.pane_manager.rebuild_display_caches_and_sync();
                // extend(visibility_events);
                Ok(())
            }
            Setting::Clipboard => {
                let enable = value == SettingValue::On;
                self.set_clipboard_enabled(enable)?;
                Ok(())
            }
            Setting::TabStop => {
                if let SettingValue::Number(width) = value {
                    self.pane_manager.set_tab_width(width);
                    // self.pane_manager.rebuild_display_caches_and_sync();
                    // extend(visibility_events);
                }
                Ok(())
            }
            Setting::ExpandTab => {
                let enable = value == SettingValue::On;
                self.pane_manager.set_expand_tab(enable);

                // If enabling expandtab, convert existing tabs to spaces
                if enable {
                    self.convert_tabs_to_spaces()?;
                    // self.pane_manager.rebuild_display_caches_and_sync();
                    // extend(visibility_events);
                }
                Ok(())
            }
            Setting::DCut => {
                let enable = value == SettingValue::On;
                self.set_dcut_enabled(enable);
                Ok(())
            }
            Setting::AutoFormat => {
                let enable = value == SettingValue::On;
                tracing::info!("=== SETTINGS MANAGER: AutoFormat ===");
                tracing::info!("Value received: {:?}, Enabling: {}", value, enable);
                self.set_autoformat_enabled(enable);
                tracing::info!("AutoFormat setting applied successfully");
                Ok(())
            }
        }
    }
}
