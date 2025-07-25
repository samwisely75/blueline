#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::model::{AppState, EditorMode, Pane};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn move_to_next_word_should_skip_to_next_word_in_line() {
        let mut buffer = RequestBuffer {
            lines: vec!["GET /api/users".to_string()],
            cursor_line: 0,
            cursor_col: 0,
            scroll_offset: 0,
        };
        // Start at beginning, should skip to 'api'
        move_to_next_word(&mut buffer).unwrap();
        assert_eq!(buffer.cursor_col, 4); // 'a' in 'api'
        // Skip to 'users'
        move_to_next_word(&mut buffer).unwrap();
        assert_eq!(buffer.cursor_col, 8); // 'u' in 'users'
    }

    #[test]
    fn move_to_next_word_should_wrap_to_next_line() {
        let mut buffer = RequestBuffer {
            lines: vec!["GET /api".to_string(), "users".to_string()],
            cursor_line: 0,
            cursor_col: 8, // end of line
            scroll_offset: 0,
        };
        move_to_next_word(&mut buffer).unwrap();
        assert_eq!(buffer.cursor_line, 1);
        assert_eq!(buffer.cursor_col, 0);
    }

    #[test]
    fn move_to_next_word_command_should_be_relevant_for_w_in_normal_mode() {
        let command = MoveToNextWordCommand;
        let mut state = AppState::new((80, 24), false);
        state.mode = EditorMode::Normal;
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(command.is_relevant(&state, &event));
    }

    #[test]
    fn move_to_next_word_command_should_not_be_relevant_in_insert_mode() {
        let command = MoveToNextWordCommand;
        let mut state = AppState::new((80, 24), false);
        state.mode = EditorMode::Insert;
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        assert!(!command.is_relevant(&state, &event));
    }

    #[test]
    fn move_to_next_word_command_should_move_cursor_in_request_pane() {
        let command = MoveToNextWordCommand;
        let mut state = AppState::new((80, 24), false);
        state.mode = EditorMode::Normal;
        state.current_pane = Pane::Request;
        state.request_buffer.lines = vec!["GET /api users".to_string()];
        state.request_buffer.cursor_line = 0;
        state.request_buffer.cursor_col = 0;
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        let result = command.process(event, &mut state).unwrap();
        assert!(result);
        assert_eq!(state.request_buffer.cursor_col, 4); // 'a' in 'api'
    }

    #[test]
    fn move_to_next_word_command_should_move_cursor_in_response_pane() {
        let command = MoveToNextWordCommand;
        let mut state = AppState::new((80, 24), false);
        state.mode = EditorMode::Normal;
        state.current_pane = Pane::Response;
        state.response_buffer = Some(ResponseBuffer::new("foo bar baz".to_string()));
        if let Some(ref mut buffer) = state.response_buffer {
            buffer.cursor_line = 0;
            buffer.cursor_col = 0;
        }
        let event = KeyEvent::new(KeyCode::Char('w'), KeyModifiers::NONE);
        let result = command.process(event, &mut state).unwrap();
        assert!(result);
        if let Some(ref buffer) = state.response_buffer {
            assert_eq!(buffer.cursor_col, 4); // 'b' in 'bar'
        }
    }
}
