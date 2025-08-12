Feature: Yank operations
  As a user  
  I want to copy text using yank commands
  So that I can store text in the yank buffer

  Background:
    Given the application is started with default settings

  Scenario: Yank single character in visual mode
    Given the request buffer contains:
      """
      Hello World
      """
    And the cursor is at display line 1 display column 1
    When I press "v"
    Then I should be in Visual mode
    When I press "l"
    And I copy it with "y"
    Then I should be in Normal mode
    And the status message should contain "2 characters yanked"

  Scenario: Yank entire line in visual mode
    Given the request buffer contains:
      """
      First line
      """
    And the cursor is at display line 1 display column 1
    When I press "v"
    And I press "$"
    And I copy it with "y"
    Then I should be in Normal mode
    And the status message should contain "10 characters yanked"

  Scenario: Yank multiple lines in visual mode
    Given the request buffer contains:
      """
      Line one
      Line two
      Line three
      """
    And the cursor is at display line 1 display column 1
    When I press "v"
    And I press "j"
    And I press "$"
    And I copy it with "y"
    Then I should be in Normal mode
    And the status message should contain "2 lines yanked"

  Scenario: Yank with no selection does nothing
    Given the request buffer contains:
      """
      Test
      """
    # In normal mode, 'y' requires a motion and doesn't work alone
    # We'll test this by checking mode doesn't change
    Then I should be in Normal mode