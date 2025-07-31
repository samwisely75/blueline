Feature: Real Application Double-byte Character Bug
  As a developer
  I want to test the actual blueline application with real terminal interaction
  So that I can identify the real cause of the rendering bug

  @real_app @double_byte_bug
  Scenario: Test real application with actual key sequence
    Given I build the blueline application
    When I launch the real blueline application
    And I send key "i" to enter insert mode
    And I type "GET _search" in the application
    And I send Escape key to exit insert mode
    And I send Enter key to execute request
    Then I should see the request pane content
    And I should see the response pane content
    And the screen should not be blacked out