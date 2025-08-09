Feature: Blueline HTTP Client REPL - Integration Tests
  As a developer
  I want to interact with HTTP APIs using a vim-style terminal interface
  So that I can efficiently test and debug web services

  # This is the main integration feature file that combines scenarios from:
  # - movement.feature: Cursor movement and navigation
  # - mode_transitions.feature: Switching between Normal/Insert/Command modes
  # - editing.feature: Text editing in insert mode (typing, backspace, special chars)
  # - command_line.feature: Colon commands for HTTP operations
  # - application.feature: Startup configuration and profiles

  Background:
    Given blueline is running with default profile
    And I am in the request pane
    And I am in normal mode

  # @integration @complete_workflow
  # Scenario: Complete HTTP request workflow
  #   Given the request buffer is empty
  #   And I am in normal mode
  #   # Mode transition: Enter insert mode
  #   When I press "i"
  #   Then I am in insert mode
  #   # Text editing: Type HTTP request
  #   When I type:
  #     """
  #     POST /api/users
  #     {"name": "John Doe", "email": "john@example.com"}
  #     """
  #   # Mode transition: Return to normal mode
  #   And I press Escape
  #   Then I am in normal mode
  #   # Movement: Navigate to check content
  #   When I press "0"
  #   Then the cursor moves to the beginning of the line
  #   When I press "j"
  #   Then the cursor moves down
  #   # Command line: Execute HTTP request
  #   When I press ":"
  #   Then I am in command mode
  #   When I type "x"
  #   And I press Enter
  #   Then the HTTP request is executed
  #   And I am in normal mode
  #   And the response appears in the response pane
  #   # Movement: Switch to response pane
  #   When I press "Ctrl+W"
  #   And I press "j"
  #   Then I am in the response pane
  #   # Movement: Navigate response
  #   When I press "j"
  #   Then the cursor moves down
  #   # Command line: Close response and quit
  #   When I press ":"
  #   Then I am in command mode
  #   When I type "q"
  #   And I press Enter
  #   Then the response pane closes
  #   And I am in the request pane
  #   When I press ":"
  #   And I type "q"
  #   And I press Enter
  #   Then the application exits

  @integration @error_handling
  Scenario: Error handling across all command categories
    Given I am in normal mode
    # Test invalid movement
    When I press "h"
    # At beginning of line, h should not move cursor
    Then the cursor does not move
    # Test mode transition with invalid command
    When I press ":"
    Then I am in command mode
    When I type "invalid_command"
    And I press Enter
    Then I see an error message "Unknown command: invalid_command"
    And I am in normal mode
    # Test canceling command mode
    When I press ":"
    Then I am in command mode
    When I press Escape
    Then I am in normal mode
    And the command buffer is cleared
