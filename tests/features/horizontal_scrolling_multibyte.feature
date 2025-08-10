Feature: Character-aware horizontal scrolling with multibyte characters
  As a developer
  I want horizontal scrolling to work on character boundaries with multibyte text
  So that I can efficiently navigate long lines with international characters

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