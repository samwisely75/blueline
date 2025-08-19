Feature: Visual Character deletion operations

  Scenario: Delete rightmost character in visual selection (Issue #137)
    Given the application is started with default settings
    And the request buffer contains:
      """
      TEST 1234
      """
    And the cursor is at display line 1 display column 1
    When I press "v"
    Then I should be in Visual mode
    When I press "l"
    When I press "l"
    When I press "l"
    When I press "l"
    When I press "l"
    Then the cursor should be at display line 1 display column 6
    When I press "d"
    Then I should be in Normal mode
    And I should see "234" in the request pane at line 1

  Scenario: Delete single character in visual mode
    Given the application is started with default settings
    And the request buffer contains:
      """
      HELLO
      """
    And the cursor is at display line 1 display column 2
    When I press "v"
    When I press "d"
    Then I should be in Normal mode
    And I should see "HLLO" in the request pane at line 1

  Scenario: Delete multi-character selection
    Given the application is started with default settings
    And the request buffer contains:
      """
      Hello World
      """
    And the cursor is at display line 1 display column 1
    When I press "v"
    When I press "l"
    When I press "l"
    When I press "l"
    When I press "l"
    When I press "d"
    Then I should be in Normal mode
    And I should see " World" in the request pane at line 1

  Scenario: Delete selection across multiple characters with multi-byte characters
    Given the application is started with default settings
    And the request buffer contains:
      """
      こんにちは World
      """
    And the cursor is at display line 1 display column 1
    When I press "v"
    When I press "l"
    When I press "l"
    When I press "d"
    Then I should be in Normal mode
    And I should see "にちは World" in the request pane at line 1