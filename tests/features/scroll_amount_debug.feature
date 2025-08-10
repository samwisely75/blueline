Feature: Debug horizontal scroll amounts with multibyte characters  
  As a developer
  I want to observe exact scroll amounts and cursor positions
  So that I can verify character-aware scrolling behavior

  Background:
    Given the application is started with default settings
    And wrap is off
    And the request buffer is empty
    And I am in the Request pane

  Scenario: Debug scroll behavior with double-byte characters
    Given I am in Insert mode
    # Type 30 double-byte chars (60 display columns) - should be visible
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよ"
    And I press Escape
    # Check initial cursor position - should be at column 58 (29 * 2)
    Then the cursor is at display line 0 display column 58
    # Now add more chars to exceed pane width
    When I press "a"
    # Type more double-byte chars to force horizontal scrolling
    And I type "らりるれろわをんあいうえおかきくけこさしすせそたちつてとなにぬねの"
    And I press Escape
    # Cursor should now be beyond visible area
    # Let's check the exact position and what happens with scrolling
    When I press "0"
    Then the cursor is at display line 0 display column 0
    # Now test scrolling behavior
    When I press "shift+Right"
    # After first scroll, check cursor position to see how much it moved
    And I press "$"
    # Going to end should show us the scrolling behavior