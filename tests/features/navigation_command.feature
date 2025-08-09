Feature: Navigation Commands
  As a user of the vim-like HTTP client
  I want to navigate text using vim-style movement commands
  So that I can efficiently move around my request content

  Background:
    Given the application is started with default settings
    And the request buffer is empty

  Scenario: Basic character movement with h/j/k/l
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "line 1"
    And I press Enter
    When I type "line 2 with more text"
    And I press Enter  
    When I type "line 3"
    And I press Escape
    Then I should be in Normal mode
    When I press "k"
    Then the cursor should move up one line
    When I press "j"
    Then the cursor should move down one line
    When I press "h"
    Then the cursor should move left one character
    When I press "l"
    Then the cursor should move right one character

  Scenario: Word movement with w/b/e
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "hello world test"
    And I press Escape
    Then I should be in Normal mode
    And the cursor should be at the end of "test"
    When I press "b"
    Then the cursor should move to the beginning of "test"
    When I press "b"
    Then the cursor should move to the beginning of "world"
    When I press "w"
    Then the cursor should move to the beginning of "test"
    When I press "e"
    Then the cursor should move to the end of "test"

  Scenario: Line movement with 0 and $
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "  hello world  "
    And I press Escape
    Then I should be in Normal mode
    When I press "0"
    Then the cursor should move to column 1
    When I press "$"
    Then the cursor should move to the end of the line

  Scenario: Navigation at buffer boundaries
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "single line"
    And I press Escape
    Then I should be in Normal mode
    When I press "h" 11 times
    Then the cursor should not move beyond the start of line
    When I press "$"
    Then the cursor should be at the end of the line
    When I press "l"
    Then the cursor should not move beyond the end of line

  Scenario: Multiple line navigation
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "first line"
    And I press Enter
    When I type "second line"
    And I press Enter
    When I type "third line"
    And I press Escape
    Then I should be in Normal mode
    When I press "k" 2 times
    Then the cursor should be on line 1
    When I press "j" 2 times
    Then the cursor should be on line 3
    When I press "j"
    Then the cursor should remain on line 3