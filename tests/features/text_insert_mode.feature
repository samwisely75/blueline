Feature: Text Insert Mode Operations
  As a developer using blueline
  I want to insert text efficiently in insert mode
  So that I can compose HTTP requests quickly

  Background:
    Given the application is started with default settings
    And the request buffer is empty

  # === BASIC INSERT MODE SCENARIOS ===
  
  Scenario: Enter insert mode and type basic text
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    And the cursor location should be at 1:1
    When I type "GET /api/users"
    Then the cursor location should be at 1:15
    And I press Escape
    Then I should be in Normal mode
    And the cursor location should be at 1:14
    And I should see "GET /api/users" in the output
    And the cursor shape should be a block

  Scenario: Insert newline with Enter key
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "GET /api/users"
    And I press Enter
    When I type "Content-Type: application/json"
    And I press Escape
    Then I should be in Normal mode
    And I should see "GET /api/users" in the output
    And I should see "Content-Type: application/json" in the output

  Scenario: Insert multiline request with newlines
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "POST /api/users"
    And I press Enter
    When I type "Content-Type: application/json"
    And I press Enter
    And I press Enter
    When I type "name: John"
    And I press Escape
    Then I should be in Normal mode
    And I should see "POST /api/users" in the output
    And I should see "Content-Type: application/json" in the output
    And I should see "John" in the output

  Scenario: Insert text with special characters
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "Special chars: @#$%^&*()_+-={}[]"
    And I press Escape
    Then I should be in Normal mode
    And I should see "Special chars: @#$%^&*()_+-={}[]" in the output

  Scenario: Insert text with quotes
    Given I am in Normal mode
    When I press "i"
    Then I should be in Insert mode
    When I type "Content-Type: application/json"
    And I press Escape
    Then I should be in Normal mode
    And I should see "Content-Type: application/json" in the output