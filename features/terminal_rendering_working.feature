Feature: Terminal Rendering Integrity
  As a user of blueline
  I want the terminal display to remain stable and responsive
  So that I can see my content and interact with the application

  Background:
    Given blueline is launched in a terminal
    And the initial screen is rendered

  Scenario: Screen remains visible after startup
    Then the screen should not be blank
    And I should see line numbers in the request pane
    And I should see the status bar at the bottom

  Scenario: Typing text keeps screen visible  
    When I enter insert mode by pressing "i"
    And I type "GET /api/test"
    Then the screen should not be blank
    And I should see "GET /api/test" in the request pane
    And the cursor should be visible

  Scenario: HTTP request execution preserves screen content
    Given I have typed a simple HTTP request
    When I execute the request by pressing Enter
    And I wait for the response
    Then the screen should not be blank
    And the request pane should still show my request
    And the response pane should show response content or error message

  Scenario: Navigation keys work without blanking screen
    Given I have some content in the request pane
    When I press "j" to move down
    And I press "k" to move up  
    And I press "h" to move left
    And I press "l" to move right
    Then the screen should not be blank
    And the cursor position should change appropriately

  Scenario: Backspace and delete keys work correctly
    Given I am in insert mode
    And I have typed some text
    When I press backspace
    Then the screen should not be blank
    And the last character should be removed
    When I press the delete key
    Then the screen should not be blank
    And the character at cursor should be removed

  Scenario: Mode switching preserves display
    Given I am in normal mode
    When I press "i" to enter insert mode
    Then the status bar should show "INSERT"
    And the screen should not be blank
    When I press Escape to return to normal mode
    Then the status bar should show "NORMAL"
    And the screen should not be blank

  Scenario: Rapid key input doesn't corrupt display
    Given I am in insert mode
    When I type rapidly "abcdefghijklmnopqrstuvwxyz" without delays
    Then the screen should not be blank
    And all typed characters should be visible
    And the cursor should be at the end of the text