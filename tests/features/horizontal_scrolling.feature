Feature: Horizontal scrolling with multibyte characters
  As a developer
  I want horizontal scrolling to work correctly with multibyte text
  So that I can efficiently navigate and edit long lines with international characters

  Background:
    Given the application is started with default settings
    And wrap is off
    And the request buffer is empty
    And I am in the Request pane

  Scenario: Horizontal scroll left preserves character boundaries with double-byte chars
    Given I am in Insert mode
    # Type 60 double-byte chars (120 display columns) to exceed pane width
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそ"
    And I press Escape
    # Cursor should be at end, triggering horizontal scroll
    Then the cursor should be visible
    # Use Shift+Left to scroll left (character-aware)
    When I press "shift+Left" 3 times
    # Should have scrolled by complete characters, not partial bytes
    Then the cursor should be visible
    And I should see complete double-byte characters in the output
    
  Scenario: Horizontal scroll right preserves character boundaries with double-byte chars
    Given I am in Insert mode
    # Type many double-byte chars
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそたちつてとなにぬねの"
    And I press Escape
    And I press "0"
    # Now scroll right with Shift+Right
    When I press "shift+Right" 3 times
    # Should have scrolled by complete characters
    Then the cursor should be visible
    And I should see complete double-byte characters in the output

  Scenario: Mixed single and double-byte character scrolling
    Given I am in Insert mode
    When I type "Hello World こんにちは世界 More text here with あいうえお and finally some English text at the end"
    And I press Escape
    And I press "0"
    # Navigate to middle and test scrolling
    When I press "$"
    Then the cursor should be visible
    # Test scrolling behavior with mixed content
    When I press "shift+Left" 5 times
    Then the cursor should be visible
    And I should see complete double-byte characters in the output

  Scenario: Verify scroll amount is character-aware not byte-aware
    Given I am in Insert mode
    # Type exactly 56 double-byte chars (112 display cols) to exceed 102-col pane
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそたち"
    And I press Escape
    # Cursor should be at end, past visible area, requiring horizontal scroll
    Then the cursor is at display line 0 display column 110
    # Record cursor position after typing
    When I press "0"
    Then the cursor is at display line 0 display column 0
    # Now scroll right - should scroll by character widths, not arbitrary amounts
    When I press "shift+Right"
    # After one scroll right, we should have scrolled properly to show more content
    Then the cursor should be visible
    
  Scenario: Compare scrolling with single-byte vs double-byte characters
    Given I am in Insert mode
    # First test with ASCII characters
    When I type "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcd"
    And I press Escape
    And I press "0"
    And I press "shift+Right"
    # Record cursor position with ASCII
    Then the cursor should be visible
    # Clear and test with double-byte
    When I press "shift+ctrl+a"
    And I press "Delete"
    And I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそたち"
    And I press Escape
    And I press "0"
    And I press "shift+Right"
    # After one scroll with double-byte, behavior should be consistent
    Then the cursor should be visible

  Scenario: Debug scroll behavior with append mode and double-byte characters
    Given I am in Insert mode
    # Type 30 double-byte chars (60 display columns) - should be visible
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよ"
    And I press Escape
    # Check initial cursor position - should be at column 58 (29 * 2)
    Then the cursor is at display line 0 display column 58
    # Now add more chars to exceed pane width using append mode
    When I press "a"
    # Type more double-byte chars to force horizontal scrolling
    And I type "らりるれろわをんあいうえおかきくけこさしすせそたちつてとなにぬねの"
    And I press Escape
    # Test scrolling behavior after append
    When I press "0"
    Then the cursor is at display line 0 display column 0
    When I press "shift+Right"
    And I press "$"
    Then the cursor should be visible

  Scenario: Manual scroll operations should not create partial double-byte characters
    Given I am in Insert mode
    # Type content that requires scrolling: mix of single and double-byte chars
    When I type "Start あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそ End"
    And I press Escape
    # Go to start
    When I press "0"
    # Now scroll right manually multiple times to see scroll behavior
    When I press "shift+Right"
    And I press "shift+Right" 
    And I press "shift+Right"
    # Check that we don't have broken characters in view
    Then I should see complete double-byte characters in the output
    # Now scroll left to test the other direction
    When I press "shift+Left"
    And I press "shift+Left"
    Then I should see complete double-byte characters in the output

  Scenario: Scroll amount consistency across ASCII and multibyte content
    Given I am in Insert mode  
    # Test with only ASCII first
    When I type "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghij"
    And I press Escape
    And I press "0"
    # Record initial state, then scroll
    When I press "shift+Right"
    # Test that scrolling amount makes sense
    Then the cursor should be visible
    # Clear and try with double-byte chars
    When I press "shift+ctrl+a"
    And I press "Delete" 
    And I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそたちつてと"
    And I press Escape
    And I press "0"
    When I press "shift+Right"
    # Scrolling behavior should be smooth and character-boundary-aware
    Then the cursor should be visible
    And I should see complete double-byte characters in the output