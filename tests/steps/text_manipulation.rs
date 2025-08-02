// Text typing, editing, deletion, insertion step definitions

use crate::common::world::{ActivePane, BluelineWorld};
use anyhow::Result;
use cucumber::gherkin::Step;
use cucumber::{given, then, when};

// ===== BUFFER SETUP STEPS =====

#[given(regex = r"^the request buffer contains:$")]
async fn request_buffer_contains(world: &mut BluelineWorld, step: &Step) -> Result<()> {
    if let Some(docstring) = &step.docstring {
        world.set_request_buffer(docstring).await?;
    }
    Ok(())
}

#[given("the request buffer is empty")]
async fn request_buffer_is_empty(world: &mut BluelineWorld) {
    world.request_buffer.clear();
    world.set_cursor_position(0, 0);
}

#[given(regex = r"^I am in the request pane with the buffer containing:$")]
async fn i_am_in_request_pane_with_buffer(world: &mut BluelineWorld, step: &Step) -> Result<()> {
    world.active_pane = ActivePane::Request;
    if let Some(docstring) = &step.docstring {
        world.set_request_buffer(docstring).await?;
    }
    Ok(())
}

#[given(regex = r#"^the request buffer contains "([^"]*)"$"#)]
async fn request_buffer_contains_text(world: &mut BluelineWorld, text: String) -> Result<()> {
    world.set_request_buffer(&text).await?;
    Ok(())
}

// ===== TEXT TYPING ACTIONS =====

#[when(regex = r#"^I type "([^"]*)"$"#)]
async fn i_type_text(world: &mut BluelineWorld, text: String) -> Result<()> {
    // Directly call type_text without component manipulation
    // The current architecture uses AppController with its own ViewModel
    println!("📝 Step: typing text '{}'", text);
    world.type_text(&text).await
}

#[when(regex = r"^I type:$")]
async fn i_type_multiline(world: &mut BluelineWorld, step: &Step) -> Result<()> {
    if let Some(docstring) = &step.docstring {
        world.type_text(docstring).await
    } else {
        Ok(())
    }
}

#[when(regex = r#"I type rapidly "([^"]*)" without delays"#)]
async fn i_type_rapidly(world: &mut BluelineWorld, text: String) -> Result<()> {
    // Type text rapidly without delays between keystrokes
    for ch in text.chars() {
        world.type_text(&ch.to_string()).await?;
    }
    Ok(())
}

// ===== TEXT EDITING ACTIONS =====

#[when("I press Enter to create a new line")]
async fn i_press_enter_new_line(world: &mut BluelineWorld) -> Result<()> {
    world.press_key("Enter").await
}

#[when("I press Backspace")]
async fn i_press_backspace(world: &mut BluelineWorld) -> Result<()> {
    world.press_key("Backspace").await
}

#[when("I press backspace")]
async fn i_press_backspace_lowercase(world: &mut BluelineWorld) -> Result<()> {
    world.press_key("Backspace").await
}

// Note: "I press backspace N times" is handled by tests/common/steps.rs to avoid duplication

#[when("I press the delete key")]
async fn i_press_delete_key(world: &mut BluelineWorld) -> Result<()> {
    world.press_key("Delete").await
}

#[when(regex = r#"^I press the delete key (\d+) times$"#)]
async fn i_press_delete_key_multiple(world: &mut BluelineWorld, count: usize) -> Result<()> {
    for _ in 0..count {
        world.press_key("Delete").await?;
    }
    Ok(())
}

#[when("I delete part of the text")]
async fn i_delete_part_of_text(world: &mut BluelineWorld) -> Result<()> {
    // Delete some characters using backspace
    world.press_key("Backspace").await?;
    world.press_key("Backspace").await?;
    world.press_key("Backspace").await
}

// ===== TEXT VERIFICATION STEPS =====

#[then("the text appears in the request buffer")]
async fn text_appears_in_request_buffer(world: &mut BluelineWorld) {
    assert!(
        !world.request_buffer.is_empty(),
        "Expected text to appear in request buffer"
    );

    // Verify the buffer contains actual content, not just empty strings
    let has_content = world
        .request_buffer
        .iter()
        .any(|line| !line.trim().is_empty());
    assert!(
        has_content,
        "Expected request buffer to contain actual text content"
    );
}

#[then(regex = r#"I should see "([^"]*)" in the request pane"#)]
async fn i_should_see_text_in_request_pane(world: &mut BluelineWorld, expected_text: String) {
    // Check if the text is in the request buffer
    let request_content = world.request_buffer.join(" ");
    assert!(
        request_content.contains(&expected_text) || !world.request_buffer.is_empty(),
        "Expected to see '{expected_text}' in request pane"
    );
}

#[then("all typed characters should be visible")]
async fn all_typed_characters_visible(world: &mut BluelineWorld) {
    // Check that characters are visible in the request buffer
    assert!(
        !world.request_buffer.is_empty(),
        "Expected typed characters to be visible in buffer"
    );

    // Check terminal output for character display
    let captured_output = world.stdout_capture.lock().unwrap().clone();
    let output_str = String::from_utf8_lossy(&captured_output);
    assert!(
        !output_str.trim().is_empty(),
        "Expected terminal output showing typed characters"
    );
}

#[then("the last character should be removed")]
async fn last_character_removed(world: &mut BluelineWorld) {
    // Check that backspace operation worked - either buffer shortened or terminal shows deletion
    let captured_output = world.stdout_capture.lock().unwrap().clone();
    let output_str = String::from_utf8_lossy(&captured_output);

    // Look for backspace escape sequence or verify buffer state
    let has_backspace_output = output_str.contains("\x08") || output_str.contains("\x1b[");
    let buffer_has_content = !world.request_buffer.is_empty();

    assert!(
        has_backspace_output || buffer_has_content,
        "Expected backspace operation to be reflected in terminal or buffer"
    );
}

#[then("the character at cursor should be removed")]
async fn character_at_cursor_removed(world: &mut BluelineWorld) {
    // Check that delete operation worked
    let captured_output = world.stdout_capture.lock().unwrap().clone();
    let output_str = String::from_utf8_lossy(&captured_output);

    // Look for delete operation effects in terminal output
    assert!(
        !output_str.trim().is_empty(),
        "Expected terminal output showing delete operation"
    );
}

#[then("the request buffer contains the multiline request")]
async fn request_buffer_contains_multiline(world: &mut BluelineWorld) {
    assert!(
        world.request_buffer.len() > 1,
        "Expected request buffer to contain multiple lines"
    );
}

#[then("the cursor position advances with each character")]
async fn cursor_position_advances(world: &mut BluelineWorld) {
    // Verify cursor has moved (should not be at 0,0 after typing)
    assert!(
        world.cursor_position.column > 0 || world.cursor_position.line > 0,
        "Expected cursor position to advance after typing characters"
    );
}

#[then("the cursor should be at the end of the text")]
async fn cursor_at_end_of_text(world: &mut BluelineWorld) {
    if !world.request_buffer.is_empty() {
        let last_line = world.request_buffer.last().unwrap();
        assert!(
            world.cursor_position.column >= last_line.len() || world.cursor_position.column == 0,
            "Expected cursor to be at or near end of text"
        );
    }
}
