Feature: Arrow Key Navigation in All Modes
  As a user of the vim-like HTTP client
  I want arrow keys to work consistently across different modes
  So that I can navigate naturally regardless of the current mode

  Background:
    Given the application is started with default settings
    And the request buffer is empty

  Scenario: Arrow keys in Normal mode
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "line 1"
    And I press Enter
    When I type "line 2 with text"
    And I press Enter
    When I type "line 3"
    And I press Escape
    Then I should be in Normal mode
    When I press the Up arrow key
    Then the cursor should move up one line
    When I press the Down arrow key
    Then the cursor should move down one line
    When I press the Left arrow key
    Then the cursor should move left one character
    When I press the Right arrow key
    Then the cursor should move right one character

  Scenario: Arrow keys in Insert mode
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "hello"
    When I press the Left arrow key
    Then the cursor should move left one character
    And I should remain in Insert mode
    When I press the Right arrow key
    Then the cursor should move right one character
    And I should remain in Insert mode
    When I type " world"
    And I press Enter
    When I type "second line"
    When I press the Up arrow key
    Then the cursor should move up one line
    And I should remain in Insert mode
    When I press the Down arrow key
    Then the cursor should move down one line
    And I should remain in Insert mode

  Scenario: Arrow keys in Visual mode
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "hello world test"
    And I press Escape
    Then I should be in Normal mode
    When I press "v"
    Then I should be in Visual mode
    When I press the Right arrow key
    Then the selection should expand
    And I should remain in Visual mode
    When I press the Right arrow key
    Then the selection should expand further
    When I press the Left arrow key
    Then the selection should contract
    And I should remain in Visual mode

  Scenario: Arrow keys behavior at buffer boundaries in Normal mode
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "single line content"
    And I press Escape
    Then I should be in Normal mode
    When I press the Up arrow key
    Then the cursor should remain on line 1
    When I press the Down arrow key
    Then the cursor should remain on line 1
    When I press "0"
    Then the cursor should move to column 1
    When I press the Left arrow key
    Then the cursor should not move beyond the start of line
    When I press "$"
    Then the cursor should move to the end of the line
    When I press the Right arrow key
    Then the cursor should not move beyond the end of line

  Scenario: Arrow keys behavior at buffer boundaries in Insert mode
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "content"
    When I press the Left arrow key 8 times
    Then the cursor should not move beyond the start of line
    And I should remain in Insert mode
    When I press the Right arrow key 8 times
    Then the cursor should be at the end of the content
    And I should remain in Insert mode

  Scenario: Mixed navigation with arrow keys and vim keys
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "mixed navigation test"
    And I press Escape
    Then I should be in Normal mode
    When I press the Left arrow key 2 times
    Then the cursor should move left
    When I press "h"
    Then the cursor should move left one character
    When I press the Right arrow key
    Then the cursor should move right one character
    When I press "l"
    Then the cursor should move right one character