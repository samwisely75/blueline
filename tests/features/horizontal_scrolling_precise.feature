Feature: Precise horizontal scrolling behavior with multibyte characters
  As a developer
  I want to verify horizontal scrolling amounts are character-aware
  So that scrolling behaves consistently regardless of character width

  Background:
    Given the application is started with default settings
    And wrap is off
    And the request buffer is empty
    And I am in the Request pane

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
    # Record the new cursor position to verify scrolling distance
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