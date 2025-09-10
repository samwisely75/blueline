Feature: Auto-format JSON responses
  As a user of blueline
  I want JSON responses to be automatically formatted
  So that I can read complex JSON data more easily

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane
    And I am in Insert mode

  Scenario: Auto-format is disabled by default
    Given I am in auto format mode
    When I submit an HTTP request
    And I receive an HTTP response with content-type "application/json"
    And the response body is "{\"key\":\"value\",\"nested\":{\"data\":123}}"
    Then I see the response in the Response pane
    And the response text is not formatted

  Scenario: Enable auto-format with set command
    When I press Escape
    Then I should be in Normal mode
    When I type ":set autoformat on"
    And I press Enter
    Then I should see "Auto-format enabled (JSON responses will be formatted)" in the status bar
    And auto-format should be enabled

  Scenario: Disable auto-format with set command
    Given auto-format is enabled
    When I press Escape
    Then I should be in Normal mode
    When I type ":set autoformat off"
    And I press Enter
    Then I should see "Auto-format disabled (JSON responses will not be formatted)" in the status bar
    And auto-format should be disabled

  Scenario: Auto-format JSON response when enabled
    Given auto-format is enabled
    And I type "GET /api/data"
    And I press Enter
    And I press Enter
    When I execute the request with Ctrl-Enter
    And I receive an HTTP response with content-type "application/json"
    And the response body is "{\"key\":\"value\",\"nested\":{\"data\":123}}"
    Then I see the response in the Response pane
    And the response text is automatically formatted
    And the response contains proper JSON indentation
    And the response contains newlines between JSON elements

  Scenario: Do not format non-JSON response when auto-format is enabled
    Given auto-format is enabled
    And I type "GET /api/text"
    And I press Enter
    And I press Enter
    When I execute the request with Ctrl-Enter
    And I receive an HTTP response with content-type "text/plain"
    And the response body is "This is plain text"
    Then I see the response in the Response pane
    And the response text is "This is plain text"
    And the response text is not formatted

  Scenario: Handle invalid JSON gracefully when auto-format is enabled
    Given auto-format is enabled
    And I type "GET /api/invalid"
    And I press Enter
    And I press Enter
    When I execute the request with Ctrl-Enter
    And I receive an HTTP response with content-type "application/json"
    And the response body is "{\"invalid\":\"json\",}"
    Then I see the response in the Response pane
    And the response text is "{\"invalid\":\"json\",}"
    And the response text is not formatted

  Scenario: Auto-format works with text/json content-type
    Given auto-format is enabled
    And I type "GET /api/legacy"
    And I press Enter
    And I press Enter
    When I execute the request with Ctrl-Enter
    And I receive an HTTP response with content-type "text/json"
    And the response body is "{\"legacy\":true,\"format\":\"text/json\"}"
    Then I see the response in the Response pane
    And the response text is automatically formatted

  Scenario: Auto-format works with content-type including charset
    Given auto-format is enabled
    And I type "GET /api/charset"
    And I press Enter
    And I press Enter
    When I execute the request with Ctrl-Enter
    And I receive an HTTP response with content-type "application/json; charset=utf-8"
    And the response body is "{\"charset\":\"utf-8\",\"test\":true}"
    Then I see the response in the Response pane
    And the response text is automatically formatted

  Scenario: Complex nested JSON is properly formatted
    Given auto-format is enabled
    And I type "GET /api/complex"
    And I press Enter
    And I press Enter
    When I execute the request with Ctrl-Enter
    And I receive an HTTP response with content-type "application/json"
    And the response body is "{\"users\":[{\"id\":1,\"name\":\"John\",\"active\":true},{\"id\":2,\"name\":\"Jane\",\"active\":false}],\"meta\":{\"total\":2,\"page\":1}}"
    Then I see the response in the Response pane
    And the response text is automatically formatted
    And the response contains proper JSON indentation
    And the formatted JSON is valid and parseable