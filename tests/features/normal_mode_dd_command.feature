Feature: Normal mode dd command for cutting entire lines

  Scenario: Cut single line with dd command
    Given I have started the application
    When I press "i" to enter Insert mode
    And I type "first line"
    And I press "Enter" 
    And I type "second line"
    And I press "Enter"
    And I type "third line"
    And I press "Escape" to enter Normal mode
    And I press "k" to move up one line
    And I press "d" followed by "d" to cut the current line
    Then the request content should be:
      """
      first line
      third line
      """
    And I should be in Normal mode
    And the cursor should be at line 1, column 0

  Scenario: Cut first line with dd command
    Given I have started the application
    When I press "i" to enter Insert mode
    And I type "line one"
    And I press "Enter"
    And I type "line two"
    And I press "Escape" to enter Normal mode
    And I press "k" to move to first line
    And I press "d" followed by "d" to cut the current line
    Then the request content should be:
      """
      line two
      """
    And I should be in Normal mode
    And the cursor should be at line 0, column 0

  Scenario: Cut last line with dd command
    Given I have started the application
    When I press "i" to enter Insert mode
    And I type "line one"
    And I press "Enter"
    And I type "line two"
    And I press "Escape" to enter Normal mode
    And I press "d" followed by "d" to cut the current line
    Then the request content should be:
      """
      line one
      """
    And I should be in Normal mode
    And the cursor should be at line 0, column 0

  Scenario: Cut only line with dd command
    Given I have started the application
    When I press "i" to enter Insert mode
    And I type "only line"
    And I press "Escape" to enter Normal mode
    And I press "d" followed by "d" to cut the current line
    Then the request content should be empty
    And I should be in Normal mode
    And the cursor should be at line 0, column 0

  Scenario: Paste cut line after dd command
    Given I have started the application
    When I press "i" to enter Insert mode
    And I type "line to cut"
    And I press "Enter"
    And I type "other line"
    And I press "Escape" to enter Normal mode
    And I press "k" to move to first line
    And I press "d" followed by "d" to cut the current line
    And I press "p" to paste after cursor
    Then the request content should be:
      """
      other line
      line to cut
      """

  Scenario: dd command maintains yank buffer type
    Given I have started the application
    When I press "i" to enter Insert mode
    And I type "test line"
    And I press "Escape" to enter Normal mode
    And I press "d" followed by "d" to cut the current line
    And I press "p" to paste after cursor
    Then the request content should be:
      """
      test line
      """

  Scenario: dd command with multi-byte characters
    Given I have started the application
    When I press "i" to enter Insert mode
    And I type "こんにちは"
    And I press "Enter"
    And I type "Hello"
    And I press "Escape" to enter Normal mode
    And I press "k" to move to first line
    And I press "d" followed by "d" to cut the current line
    Then the request content should be:
      """
      Hello
      """
    And I should be in Normal mode
    And the cursor should be at line 0, column 0

  Scenario: dd command blocked in Insert mode
    Given I have started the application
    When I press "i" to enter Insert mode
    And I type "test line"
    And I press "d" followed by "d"
    Then the request content should be:
      """
      test linedd
      """
    And I should be in Insert mode

  Scenario: dd command blocked in Response pane
    Given I have started the application
    When I press "i" to enter Insert mode
    And I type "GET http://httpbin.org/get"
    And I press "Escape" to enter Normal mode
    And I press "Enter" to send request
    And I wait for response
    And I press "Tab" to switch to Response pane
    And I press "d" followed by "d"
    Then I should be in Normal mode
    And the response content should not be empty

  Scenario: DPrefix mode timeout handling
    Given I have started the application
    When I press "i" to enter Insert mode
    And I type "test line"
    And I press "Escape" to enter Normal mode
    And I press "d" without following "d"
    And I wait 2 seconds
    And I press "x" to cut character
    Then the request content should be:
      """
      est line
      """