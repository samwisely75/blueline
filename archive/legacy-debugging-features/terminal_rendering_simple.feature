Feature: Terminal Rendering Integrity - Simple Test
  As a user of blueline
  I want the terminal display to remain stable and responsive
  So that I can see my content and interact with the application

  Background:
    Given blueline is launched in a terminal
    And the initial screen is rendered

  Scenario: Screen remains visible after startup
    Then the screen should not be blank
    And I should see line numbers in the request pane
    And I should see the status bar at the bottom