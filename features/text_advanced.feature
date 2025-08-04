Feature: Advanced Text Operations
  As a developer using blueline
  I want to use advanced editing features like undo and copy/paste
  So that I can edit HTTP requests efficiently

  Background:
    Given the scenario state is reset
    And blueline is running with default profile
    And I am in the request pane

  # === UNDO/REDO SCENARIOS ===

  Scenario: Undo functionality (if implemented)
    Given I have typed some text
    When I delete part of the text
    And I press "u" for undo
    Then the deleted text should be restored
    And the screen should not be blank

  # === COPY/PASTE SCENARIOS ===

  Scenario: Copy and paste operations (if implemented)
    Given I have text "Hello World"
    When I select the text in visual mode
    And I copy it with "y"
    And I move to a new position
    And I paste it with "p"
    Then the copied text should appear at the new position
    And the screen should not be blank