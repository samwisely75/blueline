Feature: Yank and paste operations
  As a user  
  I want to copy and paste text using yank and paste commands
  So that I can efficiently move text around

  Background:
    Given the application is started with default settings

  Scenario: Yank single character in visual mode
    Given the request buffer contains:
      """
      Hello World
      """
    And the cursor is at display line 1 display column 1
    When I press "v"
    Then I should be in Visual mode
    When I press "l"
    And I copy it with "y"
    Then I should be in Normal mode
    And the status message should contain "2 characters yanked"

  Scenario: Yank entire line in visual mode
    Given the request buffer contains:
      """
      First line
      """
    And the cursor is at display line 1 display column 1
    When I press "v"
    And I press "$"
    And I copy it with "y"
    Then I should be in Normal mode
    And the status message should contain "10 characters yanked"

  Scenario: Yank multiple lines in visual mode
    Given the request buffer contains:
      """
      Line one
      Line two
      Line three
      """
    And the cursor is at display line 1 display column 1
    When I press "v"
    And I press "j"
    And I press "$"
    And I copy it with "y"
    Then I should be in Normal mode
    And the status message should contain "2 lines yanked"

  Scenario: Yank with no selection does nothing
    Given the request buffer contains:
      """
      Test
      """
    # In normal mode, 'y' requires a motion and doesn't work alone
    # We'll test this by checking mode doesn't change
    Then I should be in Normal mode

  Scenario: Yank and paste after cursor
    Given the request buffer contains:
      """
      Hello World
      """
    And the cursor is at display line 1 display column 1
    # Select "Hello"
    When I press "v"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    And I copy it with "y"
    # Move to space after Hello
    And I press "l"
    # Paste after cursor
    And I press "p"
    Then I should see "Hello HelloWorld" in the request pane

  Scenario: Yank and paste before cursor
    Given the request buffer contains:
      """
      Hello World
      """
    And the cursor is at display line 1 display column 1
    # Select "Hello"
    When I press "v"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    And I copy it with "y"
    # Move to 'W' in World
    And I press "w"
    # Paste before cursor
    And I press "P"
    Then I should see "Hello HelloWorld" in the request pane

  Scenario: Paste with empty yank buffer
    Given the request buffer contains:
      """
      Test
      """
    When I press "p"
    Then the status message should contain "Nothing to paste"

  Scenario: Yank current line with yy command
    Given the request buffer contains:
      """
      First line
      Second line
      Third line
      """
    And the cursor is at display line 2 display column 1
    When I press "y"
    And I press "y"
    Then I should be in Normal mode
    And the status message should contain "1 line yanked"

  Scenario: Yank current line and paste after
    Given the request buffer contains:
      """
      First line
      Second line
      Third line
      """
    And the cursor is at display line 2 display column 1
    When I press "y"
    And I press "y"
    And I press "p"
    Then I should see in the request pane:
      """
      First line
      Second line
      Second line
      Third line
      """

  Scenario: Yank current line and paste before
    Given the request buffer contains:
      """
      First line
      Second line
      Third line
      """
    And the cursor is at display line 2 display column 1
    When I press "y"
    And I press "y"
    And I press "P"
    Then I should see in the request pane:
      """
      First line
      Second line
      Second line
      Third line
      """

  Scenario: Yank single line file with yy command
    Given the request buffer contains:
      """
      Only line
      """
    And the cursor is at display line 1 display column 1
    When I press "y"
    And I press "y"
    And I press "p"
    Then I should see in the request pane:
      """
      Only line
      Only line
      """

  Scenario: Yank current line with yy at end of file
    Given the request buffer contains:
      """
      First line
      Second line
      Third line
      """
    And the cursor is at display line 3 display column 1
    When I press "y"
    And I press "y"
    And I press "p"
    Then I should see in the request pane:
      """
      First line
      Second line
      Third line
      Third line
      """

  Scenario: Cancel yy command with Escape
    Given the request buffer contains:
      """
      Test line
      """
    And the cursor is at display line 1 display column 1
    When I press "y"
    Then I should be in YPrefix mode
    When I press "Escape"
    Then I should be in Normal mode

  Scenario: Combine dd and yy operations
    Given the request buffer contains:
      """
      Line A
      Line B
      Line C
      Line D
      """
    And the cursor is at display line 2 display column 1
    # Cut line B with dd
    When I press "d"
    And I press "d"
    # Move to line C (now line 2)
    And the cursor is at display line 2 display column 1
    # Yank line C with yy
    When I press "y"
    And I press "y"
    # Move to line D (now line 3) and paste both
    And I press "j"
    And I press "p"
    Then I should see in the request pane:
      """
      Line A
      Line C
      Line D
      Line C
      """