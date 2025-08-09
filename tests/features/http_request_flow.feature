Feature: HTTP Request Flow
  As a user of blueline
  I want to execute HTTP requests and see responses
  So that I can test API endpoints effectively

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane
    And I am in Insert mode

  Scenario: Simple GET request
    When I type "GET /api/health"
    And I press Enter
    And I press Enter
    When I execute the request with Ctrl-Enter
    Then the screen should not be blank
    And I should see "GET /api/health" in the request pane
    And the response pane should be visible

  Scenario: POST request with JSON body
    When I type "POST /api/users"
    And I press Enter
    And I type "Content-Type: application/json"
    And I press Enter
    And I press Enter
    And I type "{\"name\": \"test\", \"email\": \"test@example.com\"}"
    When I execute the request with Ctrl-Enter
    Then the screen should not be blank
    And I should see "POST /api/users" in the request pane
    And I should see "application/json" in the request pane
    And the response pane should be visible

  Scenario: Request with Japanese characters
    When I type "POST /api/message"
    And I press Enter
    And I type "Content-Type: application/json"
    And I press Enter
    And I press Enter
    And I type "{\"message\": \"こんにちは世界\"}"
    When I execute the request with Ctrl-Enter
    Then the screen should not be blank
    And I should see "こんにちは世界" in the request pane
    And the response pane should be visible

  Scenario: Navigate between request and response panes
    Given I have executed a request
    When I press Tab
    Then I should be in the Response pane
    When I press Tab
    Then I should be in the Request pane

  Scenario: Clear and create new request
    Given I have text "GET /old-request" in the request buffer
    When I press Escape
    And I am in Normal mode
    And I press "d"
    And I press "d"
    When I enter Insert mode
    And I type "GET /new-request"
    Then the screen should not be blank
    And I should see "GET /new-request" in the request pane
    And I should not see "GET /old-request" in the request pane

  Scenario: Request execution status indication
    Given I have text "GET /api/test" in the request buffer
    When I execute the request with Ctrl-Enter
    Then the status bar should show "Executing..."
    And the screen should not be blank

  Scenario: Invalid URL handling
    When I type "GET invalid-url"
    And I press Enter
    And I press Enter
    When I execute the request with Ctrl-Enter
    Then the screen should not be blank
    And the response pane should show an error

  Scenario: Large response scrolling
    Given I have text "GET /large-response" in the request buffer
    When I execute the request with Ctrl-Enter
    Then the screen should not be blank
    And the response pane should be visible
    When I press Tab
    And I am in the Response pane
    And I press "j"
    Then I should be able to scroll in the response pane