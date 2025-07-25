#[cfg(test)]
mod tests {
    use super::*;
    use crate::repl::model::{AppState, EditorMode, Pane};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn arrow_keys_should_move_cursor_in_all_modes_request_pane() {
        let mut state = AppState::new((80, 24), false);
        state.request_buffer.lines = vec!["abc".to_string(), "defg".to_string()];
        state.current_pane = Pane::Request;
        let modes = [EditorMode::Normal, EditorMode::Insert, EditorMode::Command, EditorMode::Visual];
        for mode in modes.iter() {
            state.mode = mode.clone();
            state.request_buffer.cursor_line = 1;
            state.request_buffer.cursor_col = 2;
            let left = KeyEvent::new(KeyCode::Left, KeyModifiers::NONE);
            let right = KeyEvent::new(KeyCode::Right, KeyModifiers::NONE);
            let up = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
            let down = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
            // Left
            let cmd = crate::repl::commands::movement::MoveCursorLeftCommand;
            assert!(cmd.is_relevant(&state, &left));
            cmd.process(left.clone(), &mut state).unwrap();
            assert_eq!(state.request_buffer.cursor_col, 1);
            // Right
            let cmd = crate::repl::commands::movement::MoveCursorRightCommand;
            assert!(cmd.is_relevant(&state, &right));
            cmd.process(right.clone(), &mut state).unwrap();
            assert_eq!(state.request_buffer.cursor_col, 2);
            // Up
            let cmd = crate::repl::commands::movement::MoveCursorUpCommand;
            assert!(cmd.is_relevant(&state, &up));
            cmd.process(up.clone(), &mut state).unwrap();
            assert_eq!(state.request_buffer.cursor_line, 0);
            // Down
            let cmd = crate::repl::commands::movement::MoveCursorDownCommand;
            assert!(cmd.is_relevant(&state, &down));
            cmd.process(down.clone(), &mut state).unwrap();
            assert_eq!(state.request_buffer.cursor_line, 1);
        }
    }
}
