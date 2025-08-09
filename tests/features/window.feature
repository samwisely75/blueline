Feature: Window Management
  As a user of the HTTP client application
  I want to manage window panes and their layout
  So that I can effectively work with requests and responses

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane

  Scenario: Switch between request and response panes
    Given I have executed a request
    And I am in Normal mode
    When I press Tab
    Then I should be in the Response pane
    And I should be in Normal mode
    When I press Tab
    Then I should be in the Request pane
    And I should be in Normal mode

  Scenario: Expand response pane with Ctrl+J
    Given I have executed a request
    And I am in Normal mode
    When I press Ctrl+J
    Then the response pane should expand
    And I should be in Normal mode

  Scenario: Shrink response pane with Ctrl+K
    Given I have executed a request
    And I am in Normal mode
    When I press Ctrl+K
    Then the response pane should shrink
    And I should be in Normal mode

  Scenario: Pane resize respects minimum sizes
    Given I have executed a request
    And I am in Normal mode
    When I press Ctrl+J repeatedly
    Then the pane sizes should respect minimum heights
    And I should be in Normal mode

  Scenario: Pane commands without response
    Given I am in the Request pane
    And no response is visible
    And I am in Normal mode
    When I press Ctrl+J
    Then nothing should change
    And I should be in Normal mode
    When I press Ctrl+K
    Then nothing should change
    And I should be in Normal mode

  Scenario: Navigate to response pane in Insert mode
    Given I have executed a request
    And I am in Insert mode
    When I press Tab
    Then I should be in the Response pane
    And I should be in Normal mode

  Scenario: Pane focus indicator
    Given I have executed a request
    And I am in the Request pane
    Then the Request pane should be highlighted
    When I press Tab
    Then the Response pane should be highlighted
    And the Request pane should not be highlighted