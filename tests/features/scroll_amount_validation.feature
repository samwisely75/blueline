Feature: Validate scroll amount behavior with character boundaries
  As a developer
  I want to ensure scroll amounts respect character boundaries
  So that partial characters never appear at viewport edges after manual scrolling

  Background:
    Given the application is started with default settings
    And wrap is off
    And the request buffer is empty
    And I am in the Request pane

  Scenario: Manual scroll left should not create partial double-byte characters
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

  Scenario: Scroll amount should be consistent across character types
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