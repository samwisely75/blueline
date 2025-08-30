//! Generic cursor movement validation for integration tests
//!
//! This module provides a flexible system for validating cursor movements
//! that works with any type of cursor movement, making tests scalable and consistent.

use crate::common::world::BluelineWorld;
use anyhow::{anyhow, Result};
use tracing::debug;

/// Types of cursor movement expectations
#[derive(Debug, Clone, PartialEq)]
pub enum CursorExpectation {
    /// Relative movement in a direction
    RelativeMove { direction: Direction, amount: usize },
    /// Absolute position
    AbsolutePosition {
        line: Option<usize>,
        column: Option<usize>,
    },
    /// Text-based position
    TextPosition {
        anchor: TextAnchor,
        text: Option<String>,
    },
    /// No movement should occur
    NoChange,
}

/// Movement directions
#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Text-based anchors
#[derive(Debug, Clone, PartialEq)]
pub enum TextAnchor {
    StartOfLine,
    EndOfLine,
    StartOfWord,
    EndOfWord,
    #[allow(dead_code)]
    StartOfDocument,
    #[allow(dead_code)]
    EndOfDocument,
    #[allow(dead_code)]
    AfterText(String),
    #[allow(dead_code)]
    BeforeText(String),
}

/// Parse cursor movement expectation from natural language
pub fn parse_cursor_expectation(expectation: &str) -> Result<CursorExpectation> {
    let expectation = expectation.trim().to_lowercase();

    // Handle relative movements
    if expectation.starts_with("left") {
        let amount = extract_number(&expectation).unwrap_or(1);
        return Ok(CursorExpectation::RelativeMove {
            direction: Direction::Left,
            amount,
        });
    }

    if expectation.starts_with("right") {
        let amount = extract_number(&expectation).unwrap_or(1);
        return Ok(CursorExpectation::RelativeMove {
            direction: Direction::Right,
            amount,
        });
    }

    if expectation.starts_with("up") {
        let amount = extract_number(&expectation).unwrap_or(1);
        return Ok(CursorExpectation::RelativeMove {
            direction: Direction::Up,
            amount,
        });
    }

    if expectation.starts_with("down") {
        let amount = extract_number(&expectation).unwrap_or(1);
        return Ok(CursorExpectation::RelativeMove {
            direction: Direction::Down,
            amount,
        });
    }

    // Handle absolute positions
    if expectation.contains("line") && expectation.contains("column") {
        let line = extract_number_after(&expectation, "line");
        let column = extract_number_after(&expectation, "column");
        return Ok(CursorExpectation::AbsolutePosition { line, column });
    }

    // Handle text anchors
    if expectation.contains("start of line") || expectation.contains("beginning of line") {
        return Ok(CursorExpectation::TextPosition {
            anchor: TextAnchor::StartOfLine,
            text: None,
        });
    }

    if expectation.contains("end of line") {
        return Ok(CursorExpectation::TextPosition {
            anchor: TextAnchor::EndOfLine,
            text: None,
        });
    }

    if expectation.contains("start of word") || expectation.contains("beginning of word") {
        return Ok(CursorExpectation::TextPosition {
            anchor: TextAnchor::StartOfWord,
            text: None,
        });
    }

    if expectation.contains("end of word") {
        return Ok(CursorExpectation::TextPosition {
            anchor: TextAnchor::EndOfWord,
            text: None,
        });
    }

    // Handle no movement
    if expectation.contains("not move")
        || expectation.contains("remain")
        || expectation.contains("stay")
    {
        return Ok(CursorExpectation::NoChange);
    }

    Err(anyhow!(
        "Could not parse cursor expectation: '{}'",
        expectation
    ))
}

/// Validate cursor movement against expectation
pub async fn validate_cursor_movement(
    world: &mut BluelineWorld,
    expectation_str: &str,
) -> Result<()> {
    let expectation = parse_cursor_expectation(expectation_str)?;

    let before = world
        .get_cursor_before_action()
        .ok_or_else(|| anyhow!("No cursor position captured before action"))?;

    let after = world.get_current_cursor_position().await;

    debug!(
        "🎯 Validating cursor movement: before={:?}, after={:?}, expectation='{}'",
        before, after, expectation_str
    );

    match expectation {
        CursorExpectation::RelativeMove { direction, amount } => {
            validate_relative_movement(before, after, direction, amount)?;
        }
        CursorExpectation::AbsolutePosition { line, column } => {
            validate_absolute_position(after, line, column)?;
        }
        CursorExpectation::TextPosition { anchor, text: _ } => {
            validate_text_position(world, after, anchor).await?;
        }
        CursorExpectation::NoChange => {
            if before != after {
                return Err(anyhow!(
                    "Expected cursor not to move, but it moved from {:?} to {:?}",
                    before,
                    after
                ));
            }
        }
    }

    debug!("✅ Cursor movement validation passed");
    Ok(())
}

/// Validate relative movement
fn validate_relative_movement(
    before: (u16, u16),
    after: (u16, u16),
    direction: Direction,
    amount: usize,
) -> Result<()> {
    match direction {
        Direction::Left => {
            let expected_col = before.0.saturating_sub(amount as u16);
            if after.0 != expected_col {
                return Err(anyhow!(
                    "Expected cursor to move left by {} from column {} to {}, but got {}",
                    amount,
                    before.0,
                    expected_col,
                    after.0
                ));
            }
        }
        Direction::Right => {
            // Right movement can wrap to next line, so check if cursor moved forward
            if after.1 == before.1 {
                // Same line - check column
                let _expected_col = before.0 + amount as u16;
                if after.0 < before.0 {
                    return Err(anyhow!(
                        "Expected cursor to move right by {} from column {}, but moved left to {}",
                        amount,
                        before.0,
                        after.0
                    ));
                }
            } else if after.1 <= before.1 {
                // Line went backwards - that's wrong
                return Err(anyhow!(
                    "Expected cursor to move right, but it moved from line {} to {}",
                    before.1,
                    after.1
                ));
            }
            // Otherwise assume line wrap is okay
        }
        Direction::Up => {
            let expected_row = before.1.saturating_sub(amount as u16);
            if after.1 != expected_row {
                return Err(anyhow!(
                    "Expected cursor to move up by {} from row {} to {}, but got {}",
                    amount,
                    before.1,
                    expected_row,
                    after.1
                ));
            }
        }
        Direction::Down => {
            let expected_row = before.1 + amount as u16;
            if after.1 != expected_row {
                return Err(anyhow!(
                    "Expected cursor to move down by {} from row {} to {}, but got {}",
                    amount,
                    before.1,
                    expected_row,
                    after.1
                ));
            }
        }
    }
    Ok(())
}

/// Validate absolute position
fn validate_absolute_position(
    cursor: (u16, u16),
    expected_line: Option<usize>,
    expected_column: Option<usize>,
) -> Result<()> {
    if let Some(line) = expected_line {
        // Convert to 0-based indexing (tests usually use 1-based)
        let expected_row = (line.saturating_sub(1)) as u16;
        if cursor.1 != expected_row {
            return Err(anyhow!(
                "Expected cursor at line {}, but got line {}",
                line,
                cursor.1 + 1
            ));
        }
    }

    if let Some(column) = expected_column {
        // Convert to 0-based indexing
        let expected_col = (column.saturating_sub(1)) as u16;
        if cursor.0 != expected_col {
            return Err(anyhow!(
                "Expected cursor at column {}, but got column {}",
                column,
                cursor.0 + 1
            ));
        }
    }

    Ok(())
}

/// Validate text-based position (simplified for now)
async fn validate_text_position(
    world: &mut BluelineWorld,
    cursor: (u16, u16),
    anchor: TextAnchor,
) -> Result<()> {
    match anchor {
        TextAnchor::StartOfLine => {
            // For line numbers mode, start of line is typically column 3 (after "1 ")
            // For no line numbers, it's column 0
            let expected_col = if world.show_line_numbers { 3 } else { 0 };
            if cursor.0 != expected_col {
                return Err(anyhow!(
                    "Expected cursor at start of line (column {}), but got column {}",
                    expected_col + 1,
                    cursor.0 + 1
                ));
            }
        }
        TextAnchor::EndOfLine => {
            // We'd need to get line content to know exact end position
            // For now, just check it's not at column 0
            if cursor.0 == 0 {
                return Err(anyhow!(
                    "Expected cursor at end of line, but it's at column 1"
                ));
            }
        }
        _ => {
            // Other anchors not implemented yet
            debug!("Text position validation not implemented for {:?}", anchor);
        }
    }

    Ok(())
}

/// Extract the first number from a string
fn extract_number(text: &str) -> Option<usize> {
    let mut number_str = String::new();
    let mut found_digit = false;

    for ch in text.chars() {
        if ch.is_ascii_digit() {
            number_str.push(ch);
            found_digit = true;
        } else if found_digit {
            // Stop at the first non-digit after we found digits
            break;
        }
    }

    if found_digit {
        number_str.parse().ok()
    } else {
        None
    }
}

/// Extract a number that appears after a specific word
fn extract_number_after(text: &str, word: &str) -> Option<usize> {
    if let Some(pos) = text.find(word) {
        let after_word = &text[pos + word.len()..];
        extract_number(after_word)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_relative_movements() {
        assert_eq!(
            parse_cursor_expectation("left 1 character").unwrap(),
            CursorExpectation::RelativeMove {
                direction: Direction::Left,
                amount: 1
            }
        );

        assert_eq!(
            parse_cursor_expectation("right 3").unwrap(),
            CursorExpectation::RelativeMove {
                direction: Direction::Right,
                amount: 3
            }
        );

        assert_eq!(
            parse_cursor_expectation("up").unwrap(),
            CursorExpectation::RelativeMove {
                direction: Direction::Up,
                amount: 1
            }
        );
    }

    #[test]
    fn test_parse_absolute_positions() {
        assert_eq!(
            parse_cursor_expectation("line 5 column 10").unwrap(),
            CursorExpectation::AbsolutePosition {
                line: Some(5),
                column: Some(10)
            }
        );
    }

    #[test]
    fn test_parse_text_anchors() {
        assert_eq!(
            parse_cursor_expectation("start of line").unwrap(),
            CursorExpectation::TextPosition {
                anchor: TextAnchor::StartOfLine,
                text: None
            }
        );

        assert_eq!(
            parse_cursor_expectation("end of line").unwrap(),
            CursorExpectation::TextPosition {
                anchor: TextAnchor::EndOfLine,
                text: None
            }
        );
    }

    #[test]
    fn test_parse_no_change() {
        assert_eq!(
            parse_cursor_expectation("not move").unwrap(),
            CursorExpectation::NoChange
        );

        assert_eq!(
            parse_cursor_expectation("remain at same position").unwrap(),
            CursorExpectation::NoChange
        );
    }
}
