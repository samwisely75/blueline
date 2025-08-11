Feature: Text Deletion Operations
  As a developer using blueline
  I want to delete text efficiently with backspace and delete keys
  So that I can edit HTTP requests accurately

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane

  # === BACKSPACE DELETION SCENARIOS ===

  Scenario: Delete character with Backspace at end of line
    Given I am in Insert mode
    And I have text "Hello World" in the request pane
    And the cursor is at the end
    When I press Backspace
    Then the last character should be removed
    And I should see "Hello Worl" in the request pane
    
  Scenario: Delete character with Backspace in middle of line
    Given I am in Insert mode
    When I type "Hello "
    And I type "World"
    And I press backspace 5 times
    Then the screen should not be blank
    And I should see "Hello " in the request pane

  Scenario: Text deletion with backspace multiple times
    Given I am in Insert mode
    And I have text "Hello World" in the request pane
    And the cursor is at the end
    When I press backspace 6 times
    Then the screen should not be blank
    And I should see "Hello" in the request pane

  Scenario: Join lines with Backspace at line start
    Given I am in Insert mode
    When I type "GET /api/users"
    And I press Enter
    And I type "second line text"
    And I press backspace 16 times
    Then the screen should not be blank
    And I should see "GET /api/users" in the request pane

  Scenario: Backspace at beginning of first line should not delete
    Given I am in Insert mode
    And I have text "Hello World" in the request pane
    And the cursor is at the beginning
    When I press Backspace
    Then no character is deleted
    And the text remains "Hello World"

  Scenario: Backspace on blank line deletes entire line
    Given I am in Insert mode
    And the request buffer contains:
      """
      GET /api/users

      {"name": "John"}
      """
    And the cursor is on the blank line (line 2)
    When I press Backspace
    Then the blank line is deleted
    And the cursor moves to the end of the previous line
    And the text becomes:
      """
      GET /api/users
      {"name": "John"}
      """

  Scenario: Backspace on consecutive blank lines deletes current blank line
    Given I am in Insert mode
    And the request buffer contains:
      """
      GET /api/users


      {"name": "John"}
      """
    And the cursor is on the second blank line (line 3)
    When I press Backspace
    Then only the current blank line is deleted
    And the cursor moves to the end of the previous line (first blank line)
    And the text becomes:
      """
      GET /api/users

      {"name": "John"}
      """

  # === DELETE KEY SCENARIOS ===

  Scenario: Text deletion with delete key
    Given I am in Insert mode
    And I have text "Hello World" in the request pane
    And the cursor is at the beginning
    When I press the delete key 6 times
    Then the screen should not be blank
    And I should see "World" in the request pane