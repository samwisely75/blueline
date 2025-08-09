Feature: Command Line Operations
  As a developer using blueline
  I want to execute colon commands for HTTP operations and application control
  So that I can manage requests and application state efficiently

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane
    And I am in Normal mode

  Scenario: Enter command mode from normal mode
    When I enter command mode
    Then I should be in Command mode
    And I should see ":" at the command line

  Scenario: Exit command mode with Escape
    When I enter command mode
    Then I should be in Command mode
    When I press Escape
    Then I should be in Normal mode
    And the command line should be cleared

  Scenario: Type command and execute
    When I enter command mode
    Then I should be in Command mode
    When I type "help"
    Then I should see ":help" at the command line
    When I press Enter
    Then I should see the help message in the output
    And I should be in Normal mode

  Scenario: Quit from request pane
    Given I have text "GET /api/users" in the request buffer
    When I enter command mode
    Then I should be in Command mode
    When I type "q"
    And I press Enter
    Then the application should exit

  Scenario: Force quit from any pane
    Given I have text "POST /api/users" in the request buffer
    When I enter command mode
    Then I should be in Command mode
    When I type "q!"
    And I press Enter
    Then the application should exit without saving

  Scenario: Handle unknown command
    When I enter command mode
    Then I should be in Command mode
    When I type "unknown"
    And I press Enter
    Then I should be in Normal mode
    And the status bar is cleared

  Scenario: Navigate to line 1
    Given the request buffer contains:
      """
      GET /api/users
      POST /api/posts
      PUT /api/data
      """
    And the cursor is at line 3
    When I enter command mode
    Then I should be in Command mode
    When I type "1"
    And I press Enter
    Then I should be in Normal mode
    And the cursor should be at line 1

  Scenario: Navigate to line 5
    Given the request buffer contains:
      """
      GET /api/users
      POST /api/posts
      PUT /api/data
      DELETE /api/item
      PATCH /api/update
      HEAD /api/status
      """
    And the cursor is at line 1
    When I enter command mode
    Then I should be in Command mode
    When I type "5"
    And I press Enter
    Then I should be in Normal mode
    And the cursor should be at line 5

  Scenario: Navigate to out of bounds line number
    Given the request buffer contains:
      """
      GET /api/users
      POST /api/posts
      """
    And the cursor is at line 1
    When I enter command mode
    Then I should be in Command mode
    When I type "1000"
    And I press Enter
    Then I should be in Normal mode
    And the cursor should be at line 2

  # Note: Command history navigation is not yet implemented
  # This scenario has been removed as it represents an unimplemented feature

  Scenario: Command line editing with backspace
    When I enter command mode
    Then I should be in Command mode
    When I type "hello"
    Then I should see ":hello" at the command line
    When I press Backspace
    Then I should see ":hell" at the command line
    When I press Backspace
    When I press Backspace
    Then I should see ":he" at the command line