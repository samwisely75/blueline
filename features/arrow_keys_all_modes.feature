Feature: Arrow key movement in all modes
  As a user
  I want to use arrow keys to move the cursor in request/response panes regardless of mode
  So that navigation is always possible

  Background:
    Given blueline is running with default profile
    And I am in the request pane

  Scenario Outline: Move cursor with arrow keys in any mode
    Given the request buffer contains:
      """
      GET /api/test
      Header: value
      """
    And I am in <mode> mode
    When I press <arrow>
    Then the cursor moves <direction>

    Examples:
      | mode    | arrow      | direction |
      | normal  | Left       | left      |
      | normal  | Right      | right     |
      | normal  | Up         | up        |
      | normal  | Down       | down      |
      | insert  | Left       | left      |
      | insert  | Right      | right     |
      | insert  | Up         | up        |
      | insert  | Down       | down      |
      | visual  | Left       | left      |
      | visual  | Right      | right     |
      | visual  | Up         | up        |
      | visual  | Down       | down      |
      | command | Left       | left      |
      | command | Right      | right     |
      | command | Up         | up        |
      | command | Down       | down      |
