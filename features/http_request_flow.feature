Feature: HTTP Request Flow
  As a user of blueline
  I want to execute HTTP requests and see responses
  So that I can test API endpoints effectively

  Background:
    Given blueline is running with default profile
    And I am in the request pane
    And I am in insert mode

  Scenario: Simple GET request execution
    Given I type a GET request:
      """
      GET /api/health
      """
    When I execute the request
    Then the screen should not be blank
    And the response pane should appear
    And I should see a status code in the status bar
    And the original request should still be visible

  Scenario: POST request with JSON body
    Given I type a POST request:
      """
      POST /api/users
      {
        "name": "test", 
        "email": "test@example.com"
      }
      """
    When I execute the request
    Then the screen should not be blank
    And the response should show the posted data
    And both panes should remain visible

  Scenario: Request with Japanese characters
    Given I type a request with Japanese text:
      """
      POST /api/message
      {
        "message": "こんにちは世界"
      }
      """
    When I execute the request
    Then the screen should not be blank
    And the Japanese characters should be visible in the request
    And the response should echo the Japanese text correctly

  Scenario: Multiple consecutive requests
    Given I execute a first request successfully
    When I clear the request pane
    And I type a second different request
    And I execute the second request
    Then the screen should not be blank
    And the new response should replace the old one
    And the request pane should show the new request

  Scenario: Network error handling
    Given I type a request to an invalid host:
      """
      GET /test
      """
    When I execute the request
    Then the screen should not be blank
    And the response pane should show an error message
    And the error should be human-readable

  Scenario: Large response handling
    Given I type a request that returns large data:
      """
      GET /json
      """
    When I execute the request
    Then the screen should not be blank
    And the response pane should show the JSON data
    And I should be able to scroll through the response
    And the request pane should remain visible

  Scenario: Request execution status indication
    Given I have typed a valid request
    When I execute the request
    Then the status bar should immediately show "Executing..."
    And the screen should not be blank during execution
    When the response arrives
    Then the status bar should show the response status code
    And the executing indicator should disappear