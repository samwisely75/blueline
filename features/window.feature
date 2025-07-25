Feature: Window Management
  As a user of the HTTP client application
  I want to manage window panes and their layout
  So that I can effectively work with requests and responses

  Background:
    Given the application is started
    And I am in the request pane

  Scenario: Switch between panes
    Given there is a response in the response pane
    And I am in normal mode
    When I press "Ctrl+W"
    And I press "j"
    Then I am in the response pane
    And I am in normal mode
    When I press "Ctrl+W"
    And I press "k"
    Then I am in the request pane
    And I am in normal mode

  Scenario: Expand response pane with Ctrl+J
    Given there is a response in the response pane
    And I am in normal mode
    And the request pane height is 10
    When I press "Ctrl+J"
    Then the response pane expands by one line
    And the request pane height decreases by one line
    And I am still in normal mode

  Scenario: Shrink response pane with Ctrl+K
    Given there is a response in the response pane
    And I am in normal mode
    And the request pane height is 8
    When I press "Ctrl+K"
    Then the response pane shrinks by one line
    And the request pane height increases by one line
    And I am still in normal mode

  Scenario: Expand response pane respects minimum request height
    Given there is a response in the response pane
    And I am in normal mode
    And the request pane height is 3
    When I press "Ctrl+J"
    Then the request pane height remains at 3
    And the response pane height remains unchanged
    And I am still in normal mode

  Scenario: Shrink response pane respects minimum response height
    Given there is a response in the response pane
    And I am in normal mode
    And the response pane height is 3
    When I press "Ctrl+K"
    Then the response pane height remains at 3
    And the request pane height remains unchanged
    And I am still in normal mode

  Scenario: Pane resize commands require response pane
    Given I am in the request pane
    And there is no response
    And I am in normal mode
    When I press "Ctrl+J"
    Then nothing happens
    And I am still in normal mode
    When I press "Ctrl+K"
    Then nothing happens
    And I am still in normal mode
