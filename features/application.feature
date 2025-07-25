Feature: Application Configuration and Startup
  As a developer using blueline
  I want to configure blueline with different startup options
  So that I can customize the HTTP client behavior for different environments

  Scenario: Start with verbose mode
    Given blueline is started with "-v" flag
    When I execute a request:
      """
      GET /api/status
      """
    Then I see detailed request information
    And I see response headers
    And I see timing information

  Scenario: Use custom profile
    Given blueline is started with "-p staging" flag
    When I execute "GET /api/status"
    Then the request uses the staging profile configuration
    And the base URL is taken from the staging profile
