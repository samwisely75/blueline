Feature: Real VTE Application Bug Test
  As a developer  
  I want to test the actual blueline application components with VTE capture
  So that I can identify the real cause of the double-byte rendering bug

  @real_vte @double_byte_bug
  Scenario: Test real application components with VTE capture
    Given I initialize the real blueline application
    And I am in the request pane
    And I am in normal mode
    # Execute the exact problematic sequence with real components
    When I press "i"
    Then I should be in insert mode using real components
    When I type "GET _search"
    Then the real view model should contain the text
    When I press "Escape"
    Then I should be in normal mode using real components
    When I press "Enter"
    Then the real application should execute HTTP request
    And I should see real terminal output
    And the VTE should capture actual rendering
    And both panes should be rendered by real components