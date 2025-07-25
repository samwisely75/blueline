Feature: Move to next word with 'w'
  As a user
  I want to skip to the next word in the buffer using 'w' in Normal mode
  So that I can navigate quickly between words

  Background:
    Given blueline is running with default profile
    And I am in the request pane
    And I am in normal mode

  Scenario: Move to next word in request buffer
    Given the request buffer contains:
      """
      GET /api users
      """
    And the cursor is at column 0
    When I press "w"
    Then the cursor moves to column 4
    When I press "w"
    Then the cursor moves to column 8

  Scenario: Move to next word wraps to next line
    Given the request buffer contains:
      """
      GET /api
      users
      """
    And the cursor is at column 8
    When I press "w"
    Then the cursor moves to line 1 column 0

  Scenario: Move to next word in response buffer
    Given there is a response in the response pane from:
      """
      foo bar baz
      """
    And I am in the response pane
    And the cursor is at column 0
    When I press "w"
    Then the cursor moves to column 4
    When I press "w"
    Then the cursor moves to column 8
