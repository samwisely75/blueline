Feature: Text append mode with double-byte characters and horizontal scrolling
  As a developer
  I want text append mode (A command) to work correctly with double-byte characters
  So that I can append text at line end even when horizontal scrolling is required

  Background:
    Given the application is started with default settings
    And wrap is off
    And the request buffer is empty
    And I am in the Request pane
    And the pane width is set to 112

  Scenario: A command works initially with 53 double-byte characters
    Given I am in Insert mode
    # Type exactly 53 double-byte chars as described in issue #82
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさし"
    And I press Escape
    When I press "0"
    And I press "A"
    # This should work correctly - cursor at end for appending
    Then the cursor should be visible
    When I type "ab"
    And I press Escape
    # Now we have 53 DB chars + "ab" = 55 total characters
    
  Scenario: A command fails after appending to extended line (the actual bug)
    Given I am in Insert mode
    # Type exactly 53 double-byte chars (106 display columns)
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさし"
    And I press Escape
    # Cursor should be positioned at the end of the line with "し" visible
    Then the cursor should be visible
    And I should see "し" in the output
    When I press "0"
    And I press "A"
    When I type "ab"
    And I press Escape
    # Now line has 55 chars: 53 DB + 2 ASCII = 106 + 2 = 108 display columns
    # Cursor should be positioned after "ab"
    Then the cursor should be visible
    And I should see "ab" in the output
    # This next A command should go to end of all 55 chars
    When I press "0"
    And I press "A"
    # BUG CHECK: If cursor goes to "head of 53rd char" it would be at column 104
    # CORRECT: Should be at end (after 55th char) which is column 108
    # Let's see where it actually goes by typing and checking the result
    When I type "cd"
    # If bug exists: "cd" gets inserted in middle, breaking the "ab" sequence
    # If working: "cd" gets appended after "ab" creating "abcd" 
    Then I should see "abcd" in the output

  Scenario: Append mode at end of long double-byte line should scroll and show cursor
    Given I am in Insert mode
    # Type 55 double-byte chars (110 display columns) - just under 112-column pane width
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそたちつてとなにぬ"
    And I press Escape
    # Should be at end of line, cursor visible
    Then the cursor should be visible
    # Now use append mode (A) to add more characters at end
    When I press "A"
    Then I should be in Insert mode
    # Cursor should be positioned at the end, ready for appending
    And the cursor should be visible
    # Type additional double-byte characters that will require horizontal scrolling
    When I type "ねのはひふへほ"
    # New characters should be visible and cursor should scroll horizontally
    Then the cursor should be visible
    And I should see "ほ" in the output
    # Test cursor navigation still works correctly
    When I press Escape
    Then I should be in Normal mode
    When I press "0"
    Then the cursor should be visible
    When I press "$"
    # Should be able to navigate to the new end including appended characters
    Then the cursor should be visible
    And I should see "ほ" in the output

  Scenario: Append mode with mixed ASCII and double-byte characters
    Given I am in Insert mode
    # Create a line with mixed content that approaches pane width
    When I type "Start こんにちは世界 Middle あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよ"
    And I press Escape
    # Use append mode to add more content
    When I press "A"
    Then I should be in Insert mode
    And the cursor should be visible
    # Append more mixed content
    When I type " End らりるれろわをん"
    Then the cursor should be visible
    And I should see "ん" in the output
    # Verify navigation still works
    When I press Escape
    And I press "0"
    When I press "$"
    Then the cursor should be visible

  Scenario: Append mode should handle character visibility correctly
    Given I am in Insert mode
    # Type exactly enough to require scrolling when appending
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそた"
    And I press Escape
    # Use append mode
    When I press "A"
    Then I should be in Insert mode
    # Any characters typed should be immediately visible
    When I type "ち"
    Then I should see "ち" in the output
    And the cursor should be visible
    # Continue appending
    When I type "つてと"
    Then I should see "と" in the output
    And the cursor should be visible

  Scenario: Append mode cursor positioning should be consistent
    Given I am in Insert mode
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそたちつ"
    And I press Escape
    # Record position before append mode
    Then the cursor should be visible
    # Enter append mode
    When I press "A"
    # Cursor should move to after the last character for appending
    Then the cursor should be visible
    # Type a character - should appear immediately after existing content
    When I type "て"
    Then the cursor should be visible
    # Exit and navigate back to verify content was appended correctly  
    When I press Escape
    And I press "0"
    When I press "$"
    Then I should see "て" in the output

  Scenario: A command horizontal scrolling with exactly 54 double-byte characters
    Given I am in Insert mode
    # Type exactly 54 double-byte characters ending with し - requires horizontal scrolling
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそたちつてとし"
    And I press Escape
    # Go to beginning to test horizontal scrolling from start
    When I press "0"
    Then the cursor should be visible
    And the line starts with "あ"
    # A command should scroll enough to show し completely and position cursor after it
    When I press "A"
    Then I should be in Insert mode
    # After scrolling, line should start with a character beyond the pane width
    And the line starts with "か"
    And the cursor should be visible
    # Critical: し should be fully visible after horizontal scrolling
    And I should see "し" in the output

  Scenario: Dollar command horizontal scrolling with exactly 54 double-byte characters  
    Given I am in Insert mode
    # Type exactly 54 double-byte characters ending with し
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをんあいうえおかきくけこさしすせそたちつてとし"
    And I press Escape
    # Go to beginning to test horizontal scrolling from start
    When I press "0"
    Then the cursor should be visible
    And the line starts with "あ"
    # $ command should scroll enough to show し completely and position cursor on it
    When I press "$"
    # After scrolling, line should start with a character beyond the pane width (for $ command may be different)
    And the cursor should be visible
    # Critical: し should be fully visible after horizontal scrolling
    And I should see "し" in the output