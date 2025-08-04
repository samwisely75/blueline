Feature: Text Navigation Operations
  As a developer using blueline
  I want to navigate through text efficiently with vim-like commands
  So that I can move around HTTP requests quickly

  Background:
    Given the scenario state is reset
    And blueline is running with default profile
    And I am in the request pane

  # === LINE NAVIGATION SCENARIOS ===

  Scenario: Line navigation with j/k keys
    Given I am in normal mode
    And I have multiple lines of text:
      """
      GET /api/users
      Accept: application/json
      Content-Type: application/json
      """
    When I press "j" to move down
    Then the cursor should be on line 2
    And the screen should not be blank
    When I press "k" to move up
    Then the cursor should be on line 1
    And the screen should not be blank

  # === CHARACTER NAVIGATION SCENARIOS ===

  Scenario: Character navigation with h/l keys
    Given I am in normal mode
    And I have text "Hello World" on one line
    And the cursor is at the beginning
    When I press "l" 6 times
    Then the cursor should be after "Hello "
    And the screen should not be blank
    When I press "h" 3 times
    Then the cursor should be after "Hel"
    And the screen should not be blank

  # === WORD NAVIGATION SCENARIOS ===

  Scenario: Word-based navigation
    Given I am in normal mode
    And I have text "The quick brown fox jumps"
    And the cursor is at the beginning
    When I press "w" to move to next word
    Then the cursor should be at "quick"
    And the screen should not be blank
    When I press "b" to move to previous word
    Then the cursor should be at "The"
    And the screen should not be blank

  # === LINE BOUNDARY NAVIGATION SCENARIOS ===

  Scenario: Line beginning and end navigation
    Given I am in normal mode
    And I have text "Hello World" on one line
    And the cursor is in the middle
    When I press "0" to go to line beginning
    Then the cursor should be at the start of the line
    And the screen should not be blank
    When I press "$" to go to line end
    Then the cursor should be at the end of the line
    And the screen should not be blank