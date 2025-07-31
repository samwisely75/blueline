Feature: Command Line Operations
  As a developer using blueline
  I want to execute colon commands for HTTP operations and application control
  So that I can manage requests and application state efficiently

  Background:
    Given blueline is running with default profile
    And I am in the request pane
    And I am in normal mode

  # Scenario: Execute HTTP request
  #   Given I am in the request pane with the buffer containing:
  #     """
  #     GET /api/users
  #     """
  #   And I am in normal mode
  #   When I press ":"
  #   Then I am in command mode
  #   When I type "x"
  #   And I press Enter
  #   Then the HTTP request is executed
  #   And I am in normal mode
  #   And the response appears in the response pane
  #   And I can see the status code

  Scenario: Quit from request pane
    Given I am in the request pane with the buffer containing:
      """
      GET /api/users
      """
    And I am in normal mode
    When I press ":"
    Then I am in command mode
    When I type "q"
    And I press Enter
    Then the application exits

  # Scenario: Close response pane
  #   Given there is a response in the response pane from:
  #     """
  #     GET /api/users
  #     """
  #   And I am in the response pane
  #   And I am in normal mode
  #   When I press ":"
  #   Then I am in command mode
  #   When I type "q"
  #   And I press Enter
  #   Then the response pane closes
  #   And I am in the request pane
  #   And I am in normal mode
  #   And the request pane is maximized

  Scenario: Force quit from any pane
    Given I am in the request pane with the buffer containing:
      """
      POST /api/users

      {"name": "John"}
      """
    And I am in normal mode
    When I press ":"
    Then I am in command mode
    When I type "q!"
    And I press Enter
    Then the application exits without saving

  Scenario: Handle unknown command
    Given I am in the request pane with the buffer containing:
      """
      GET /api/users
      """
    And I am in normal mode
    When I press ":"
    Then I am in command mode
    When I type "unknown"
    And I press Enter
    Then I see an error message "Unknown command: unknown"
    And I am in normal mode

  # Scenario: Execute multiline HTTP request with command
  #   Given the request buffer contains:
  #     """
  #     POST /api/users
  #
  #     {"name": "John Doe"}
  #     """
  #   And I am in normal mode
  #   When I press ":"
  #   Then I am in command mode
  #   When I type "x"
  #   And I press Enter
  #   Then the POST request is executed with the JSON body
  #   And I am in normal mode

  Scenario: Navigate to line 0 should be ignored (edge case)
    Given the request buffer contains:
      """
      GET /api/users
      POST /api/posts
      """
    And I am in normal mode
    And my cursor is at line 2, column 5
    When I press ":"
    Then I am in command mode
    When I type "0"
    And I press Enter
    Then I am in normal mode
    And my cursor is at line 2, column 5

  Scenario: Navigate to line 1 (minimum valid line)
    Given the request buffer contains:
      """
      GET /api/users
      POST /api/posts
      PUT /api/data
      """
    And I am in normal mode
    And my cursor is at line 3, column 5
    When I press ":"
    Then I am in command mode
    When I type "1"
    And I press Enter
    Then I am in normal mode
    And my cursor is at line 1, column 0

  Scenario: Navigate to line 5 (many - within bounds)
    Given the request buffer contains:
      """
      GET /api/users
      POST /api/posts
      PUT /api/data
      DELETE /api/item
      PATCH /api/update
      HEAD /api/status
      """
    And I am in normal mode
    And my cursor is at line 1, column 0
    When I press ":"
    Then I am in command mode
    When I type "5"
    And I press Enter
    Then I am in normal mode
    And my cursor is at line 5, column 0

  Scenario: Navigate to line 1000 (out of bounds - should clamp to last line)
    Given the request buffer contains:
      """
      GET /api/users
      POST /api/posts
      """
    And I am in normal mode
    And my cursor is at line 1, column 0
    When I press ":"
    Then I am in command mode
    When I type "1000"
    And I press Enter
    Then I am in normal mode
    And my cursor is at line 2, column 0
