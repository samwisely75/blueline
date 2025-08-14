//! # Settings Management
//!
//! Handles settings changes from ex commands.

use crate::repl::commands::{Setting, SettingValue};
use crate::repl::events::ViewEvent;
use crate::repl::view_models::core::ViewModel;
use anyhow::Result;

impl ViewModel {
    /// Apply a setting change from an ex command
    pub fn apply_setting(&mut self, setting: Setting, value: SettingValue) -> Result<()> {
        match setting {
            Setting::Wrap => {
                let enable = value == SettingValue::On;
                self.pane_manager.set_wrap_enabled(enable);
                let visibility_events = self.pane_manager.rebuild_display_caches_and_sync();
                let mut events = vec![ViewEvent::FullRedrawRequired];
                events.extend(visibility_events);
                let _ = self.emit_view_event(events);
                Ok(())
            }
            Setting::LineNumbers => {
                let enable = value == SettingValue::On;
                self.pane_manager.set_line_numbers_visible(enable);
                let visibility_events = self.pane_manager.rebuild_display_caches_and_sync();
                let mut events = vec![ViewEvent::FullRedrawRequired];
                events.extend(visibility_events);
                let _ = self.emit_view_event(events);
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
                    let visibility_events = self.pane_manager.rebuild_display_caches_and_sync();
                    let mut events = vec![ViewEvent::FullRedrawRequired];
                    events.extend(visibility_events);
                    let _ = self.emit_view_event(events);
                }
                Ok(())
            }
        }
    }
}
