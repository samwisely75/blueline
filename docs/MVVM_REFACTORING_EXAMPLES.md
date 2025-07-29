# MVVM Refactoring: Concrete Examples

## Example 1: MoveCursorUp Command Refactoring

### Current Implementation (MVC with View Logic)
```rust
// src/repl/commands/movement.rs
impl Command for MoveCursorUpCommand {
    fn process(&self, state: &mut AppState) -> Result<()> {
        let (buffer, cache, pane_height) = match state.current_pane {
            Pane::Request => (
                &mut state.request_buffer,
                state.cache_manager.get_request_cache(),
                state.get_request_pane_height(),
            ),
            Pane::Response => (
                &mut state.response_buffer,
                state.cache_manager.get_response_cache(),
                state.get_response_pane_height(),
            ),
        };

        // Get current logical position
        let current_logical_line = buffer.cursor_line;
        let current_logical_col = buffer.cursor_col;

        // Convert to display position
        if let Some((current_display_line, current_display_col)) =
            cache.logical_to_display_position(current_logical_line, current_logical_col)
        {
            // Move up one display line
            if current_display_line > 0 {
                let target_display_line = current_display_line - 1;
                
                // Convert back to logical position
                if let Some((new_logical_line, new_logical_col)) =
                    cache.display_to_logical_position(target_display_line, current_display_col)
                {
                    buffer.cursor_line = new_logical_line;
                    buffer.cursor_col = new_logical_col;
                    
                    // Update display position
                    buffer.display_cursor_line = target_display_line;
                    buffer.display_cursor_col = current_display_col;
                    
                    // Auto-scroll if needed
                    if target_display_line < buffer.display_scroll_offset {
                        buffer.display_scroll_offset = target_display_line;
                    }
                }
            }
        }

        Ok(())
    }
}
```

### Refactored Implementation (MVVM)

#### Pure Command (No View Logic)
```rust
// src/repl/commands/movement.rs
impl Command for MoveCursorUpCommand {
    fn process(&self, model: &mut PureModel, event_bus: &mut dyn EventBus) -> Result<()> {
        let pane = model.current_pane;
        let current_pos = model.get_cursor(pane);
        
        // Pure logical operation
        if current_pos.line > 0 {
            let new_pos = LogicalPosition {
                line: current_pos.line - 1,
                column: current_pos.column,
            };
            
            // Just update model and emit event
            let event = model.move_cursor(pane, new_pos);
            event_bus.publish(event);
        }
        
        Ok(())
    }
}
```

#### ViewModel Handles Display Logic
```rust
// src/repl/view_model.rs
impl ViewModel {
    fn handle_cursor_moved(&mut self, pane: Pane, old_pos: LogicalPosition, new_pos: LogicalPosition) {
        let display_state = self.get_display_state_mut(pane);
        let cache = self.get_display_cache(pane);
        
        // Convert logical to display position
        if let Some(display_pos) = cache.logical_to_display_position(new_pos.line, new_pos.column) {
            let old_display_line = display_state.display_cursor.line;
            
            // Update display cursor
            display_state.display_cursor = DisplayPosition {
                line: display_pos.0,
                column: display_pos.1,
            };
            
            // Check if scrolling needed
            if display_pos.0 < display_state.scroll_offset {
                let old_offset = display_state.scroll_offset;
                display_state.scroll_offset = display_pos.0;
                
                self.emit_event(ViewModelEvent::ScrollPositionChanged {
                    pane,
                    old_offset,
                    new_offset: display_pos.0,
                });
            }
            
            // Emit cursor update event
            self.emit_event(ViewModelEvent::CursorRepositionRequired);
        }
    }
}
```

## Example 2: Word Movement with Auto-scroll

### Current Implementation
```rust
// src/repl/commands/movement.rs
pub fn move_to_next_word(buffer: &mut dyn Buffer, visible_height: usize) -> bool {
    let lines = buffer.lines();
    let mut line_idx = buffer.cursor_line();
    let mut col_idx = buffer.cursor_col();
    let mut scroll_occurred = false;

    // Complex word-finding logic mixed with scrolling logic
    'outer: loop {
        if line_idx >= lines.len() {
            break;
        }

        let line = &lines[line_idx];
        let chars: Vec<char> = line.chars().collect();

        // Skip current word/non-word characters
        while col_idx < chars.len() 
            && chars[col_idx].is_alphanumeric() == chars.get(col_idx + 1).map_or(false, |c| c.is_alphanumeric()) 
        {
            col_idx += 1;
        }

        // Find next word
        for i in col_idx..chars.len() {
            if chars[i].is_alphanumeric() {
                *buffer.cursor_line_mut() = line_idx;
                *buffer.cursor_col_mut() = i;
                
                // Auto-scroll if cursor moved outside visible area
                if line_idx < *buffer.scroll_offset_mut() {
                    *buffer.scroll_offset_mut() = line_idx;
                    scroll_occurred = true;
                } else if line_idx >= *buffer.scroll_offset_mut() + visible_height {
                    *buffer.scroll_offset_mut() = line_idx - visible_height + 1;
                    scroll_occurred = true;
                }
                
                break 'outer;
            }
        }

        // Move to next line
        line_idx += 1;
        col_idx = 0;
    }
    
    scroll_occurred
}
```

### Refactored Implementation

#### Pure Command
```rust
// src/repl/commands/movement.rs
impl Command for MoveToNextWordCommand {
    fn process(&self, model: &mut PureModel, event_bus: &mut dyn EventBus) -> Result<()> {
        let pane = model.current_pane;
        let content = model.get_content(pane);
        let current_pos = model.get_cursor(pane);
        
        // Pure word-finding logic only
        if let Some(next_word_pos) = find_next_word(&content, current_pos) {
            let event = model.move_cursor(pane, next_word_pos);
            event_bus.publish(event);
        }
        
        Ok(())
    }
}

// Pure function for finding next word
fn find_next_word(content: &BufferContent, from: LogicalPosition) -> Option<LogicalPosition> {
    let lines = content.lines();
    let mut line_idx = from.line;
    let mut col_idx = from.column;
    
    loop {
        if line_idx >= lines.len() {
            return None;
        }
        
        let line = &lines[line_idx];
        let chars: Vec<char> = line.chars().collect();
        
        // Skip current word
        while col_idx < chars.len() && chars[col_idx].is_alphanumeric() {
            col_idx += 1;
        }
        
        // Skip non-word characters
        while col_idx < chars.len() && !chars[col_idx].is_alphanumeric() {
            col_idx += 1;
        }
        
        // Found word on current line
        if col_idx < chars.len() {
            return Some(LogicalPosition {
                line: line_idx,
                column: col_idx,
            });
        }
        
        // Move to next line
        line_idx += 1;
        col_idx = 0;
    }
}
```

#### ViewModel Auto-scroll
```rust
// src/repl/view_model.rs
impl ViewModel {
    fn handle_cursor_moved(&mut self, pane: Pane, old_pos: LogicalPosition, new_pos: LogicalPosition) {
        // ... display position calculation ...
        
        // Auto-scroll logic centralized here
        self.ensure_cursor_visible(pane);
    }
    
    fn ensure_cursor_visible(&mut self, pane: Pane) {
        let display_state = self.get_display_state_mut(pane);
        let visible_height = self.get_visible_height(pane);
        let cursor_line = display_state.display_cursor.line;
        
        let mut events = vec![];
        
        // Scroll up if needed
        if cursor_line < display_state.scroll_offset {
            let old_offset = display_state.scroll_offset;
            display_state.scroll_offset = cursor_line;
            
            events.push(ViewModelEvent::ScrollPositionChanged {
                pane,
                old_offset,
                new_offset: cursor_line,
            });
        }
        // Scroll down if needed
        else if cursor_line >= display_state.scroll_offset + visible_height {
            let old_offset = display_state.scroll_offset;
            let new_offset = cursor_line - visible_height + 1;
            display_state.scroll_offset = new_offset;
            
            events.push(ViewModelEvent::ScrollPositionChanged {
                pane,
                old_offset,
                new_offset,
            });
        }
        
        // Emit all events
        for event in events {
            self.emit_event(event);
        }
    }
}
```

## Example 3: Insert Text with Display Update

### Current Implementation
```rust
// src/repl/commands/editing.rs
impl Command for InsertTextCommand {
    fn process(&self, state: &mut AppState) -> Result<()> {
        if state.mode != EditorMode::Insert {
            return Ok(());
        }

        match state.current_pane {
            Pane::Request => {
                let line_idx = state.request_buffer.cursor_line;
                let col_idx = state.request_buffer.cursor_col;
                
                // Insert text
                if line_idx < state.request_buffer.lines.len() {
                    state.request_buffer.lines[line_idx].insert_str(col_idx, &self.text);
                    state.request_buffer.cursor_col += self.text.len();
                    
                    // Update display cache
                    if let Err(e) = state.update_display_cache_auto() {
                        eprintln!("Failed to update display cache: {}", e);
                    }
                    
                    // Handle scrolling
                    let visible_height = state.get_request_pane_height();
                    if state.request_buffer.display_cursor_line >= 
                       state.request_buffer.display_scroll_offset + visible_height {
                        state.request_buffer.display_scroll_offset = 
                            state.request_buffer.display_cursor_line - visible_height + 1;
                    }
                }
            }
            // Similar for Response pane...
        }
        
        Ok(())
    }
}
```

### Refactored Implementation

#### Pure Command
```rust
// src/repl/commands/editing.rs
impl Command for InsertTextCommand {
    fn process(&self, model: &mut PureModel, event_bus: &mut dyn EventBus) -> Result<()> {
        if model.mode != EditorMode::Insert {
            return Ok(());
        }
        
        let pane = model.current_pane;
        let pos = model.get_cursor(pane);
        
        // Pure text insertion
        model.insert_text_at(pane, pos, &self.text);
        
        // Update cursor position
        let new_pos = LogicalPosition {
            line: pos.line,
            column: pos.column + self.text.len(),
        };
        
        // Emit events
        event_bus.publish(ModelEvent::TextInserted {
            pane,
            position: pos,
            text: self.text.clone(),
        });
        
        event_bus.publish(model.move_cursor(pane, new_pos));
        
        Ok(())
    }
}
```

#### Model Pure Text Operation
```rust
// src/repl/model/pure_model.rs
impl PureModel {
    pub fn insert_text_at(&mut self, pane: Pane, pos: LogicalPosition, text: &str) {
        let content = match pane {
            Pane::Request => &mut self.request_content,
            Pane::Response => &mut self.response_content,
        };
        
        if pos.line < content.lines.len() {
            content.lines[pos.line].insert_str(pos.column, text);
        }
    }
}
```

#### ViewModel Updates Display
```rust
// src/repl/view_model.rs
impl ViewModel {
    fn handle_text_inserted(&mut self, pane: Pane, position: LogicalPosition, text: String) {
        // Update display cache in background
        self.schedule_display_cache_update(pane);
        
        // Immediate partial update for responsiveness
        self.update_line_wrapping(pane, position.line);
        
        // Cursor will be repositioned by subsequent CursorMoved event
        
        // Emit appropriate view events
        self.emit_event(ViewModelEvent::PaneRedrawRequired { pane });
    }
    
    fn schedule_display_cache_update(&mut self, pane: Pane) {
        // Use existing background thread mechanism
        match pane {
            Pane::Request => {
                self.display_cache.schedule_request_update(
                    self.get_content_snapshot(pane),
                    self.terminal_size.0 as usize,
                );
            }
            Pane::Response => {
                // Response is synchronous for now
                self.display_cache.update_response_cache(
                    self.get_content_snapshot(pane),
                    self.terminal_size.0 as usize,
                );
            }
        }
    }
}
```

## Key Differences Summary

### Before (MVC)
1. Commands handle display logic
2. Commands calculate scroll positions
3. Commands update display cache
4. Model stores display state
5. Tight coupling between layers

### After (MVVM)
1. Commands only modify logical state
2. ViewModel handles all display calculations
3. Display cache owned by ViewModel
4. Model is pure data
5. Event-driven communication

### Benefits Demonstrated
1. **Testability**: Can test word-finding logic without any display concerns
2. **Reusability**: Same command works with different view implementations
3. **Maintainability**: Display logic centralized in ViewModel
4. **Performance**: Display updates can be optimized independently
5. **Flexibility**: Easy to add features like smooth scrolling or animations