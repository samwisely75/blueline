Feature: Advanced Text Operations
  As a developer using blueline
  I want to use advanced editing features like undo and copy/paste
  So that I can edit HTTP requests efficiently

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane
    And I am in Normal mode

  # === UNDO/REDO SCENARIOS ===

  Scenario: Undo functionality (if implemented)
    Given I have text "Hello World" in the request pane
    When I press "i"
    Then I should be in Insert mode
    When I type " test"
    And I press Escape
    Then I should be in Normal mode
    When I press "u"
    Then I should see "Hello World" in the request pane
    And I should not see "Hello World test" in the request pane

  # === COPY/PASTE SCENARIOS ===

  Scenario: Copy and paste operations (if implemented)
    Given I have text "Hello World" in the request pane
    When I press "v"
    Then I should be in Visual mode
    When I press "l" 5 times
    And I press "y"
    Then I should be in Normal mode
    When I press "$"
    And I press "p"
    Then I should see "Hello WorldHello " in the request pane