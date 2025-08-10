Feature: Edge Cases for Dollar Sign with Double-Byte Characters
  As a developer
  I want to ensure dollar sign positioning is correct with double-byte characters
  So that cursor navigation works properly with international text

  Background:
    Given the application is started with default settings
    And wrap is off
    And the request buffer is empty
    And I am in the Request pane

  Scenario: Dollar sign positions at last character not last byte
    Given I am in Insert mode
    # Type exactly 56 double-byte chars to fill 112 columns exactly
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさし"
    And I press Escape
    # Cursor should be at the 55th character (0-indexed) after typing 56 chars
    Then the cursor is at display line 0 display column 110
    When I press "0"
    Then the cursor is at display line 0 display column 0
    When I press "$"
    # Should position at the last character (55th, 0-indexed)
    # Display column should be 110 (55 * 2 = 110)
    Then the cursor is at display line 0 display column 110
    # Verify we're actually at the last character "し"
    And I should see "し" in the output

  Scenario: Dollar sign with single double-byte character at line end
    Given I am in Insert mode
    When I type "Hello World あ"
    And I press Escape
    # After typing, cursor is at position after "あ"
    When I press "0"
    Then the cursor is at display line 0 display column 0
    When I press "$"
    # Should be at the "あ" character
    # "Hello World " = 12 display columns, "あ" starts at 12, width 2
    Then the cursor is at display line 0 display column 12
    
  Scenario: Dollar sign moves correctly after editing double-byte text
    Given I am in Insert mode
    When I type "Test line with あい"
    And I press Escape
    When I press "0"
    When I press "$"
    # Should be at the last character "い"
    Then I should see "い" in the output
    When I press "0"
    Then the cursor is at display line 0 display column 0
    When I press "$"
    # Verify $ consistently goes to the same position
    Then I should see "い" in the output