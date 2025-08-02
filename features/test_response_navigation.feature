Feature: Response Pane Navigation Test
  As a developer testing blueline navigation
  I want to verify response pane navigation works correctly
  So that I can ensure all issues are fixed

  Background:
    Given blueline is running with default profile
    And I am in normal mode

  Scenario: Word navigation in response pane works correctly
    Given there is a response in the response pane from:
      """
      Hello world this is a test
      """
    And I am in the response pane
    And the cursor is at column 0
    When I press "w"
    Then the cursor moves to column 6
    When I press "w"
    Then the cursor moves to column 12
    When I press "b"
    Then the cursor moves to column 6
    When I press "b"
    Then the cursor moves to column 0

  Scenario: Vertical navigation clamps to line length in response pane
    Given there is a response in the response pane from:
      """
      This is a very long line with many words
      Short
      Another long line here
      """
    And I am in the response pane
    And the cursor is at line 0 column 35
    When I press "j"
    Then the cursor is at line 1 column 5
    When I press "j"  
    Then the cursor is at line 2 column 23

  Scenario: Japanese text navigation in response pane
    Given there is a response in the response pane from:
      """
      こんにちは world
      """
    And I am in the response pane
    And the cursor is at column 0
    When I press "w"
    Then the cursor moves to column 5
    When I press "w"
    Then the cursor moves to column 11

  Scenario: Long Japanese text does not crash application
    Given there is a response in the response pane from:
      """
      これはとても長い日本語のテキストです。プログラミングにおいて、文字エンコーディングは重要な概念です。特にダブルバイト文字を扱う際には、適切な処理が必要になります。このテストは、長い日本語テキストが正しくレンダリングされ、アプリケーションがクラッシュしないことを確認するためのものです。
      """
    And I am in the response pane
    And the cursor is at column 0
    When I press "w"
    Then the terminal state should be valid
    When I press "l" 50 times
    Then the terminal state should be valid
    When I press "h" 20 times
    Then the terminal state should be valid
    And the response pane should display content

  Scenario: Mixed ASCII and Japanese text handles navigation properly
    Given there is a response in the response pane from:
      """
      HTTP/1.1 200 OK
      Content-Type: application/json; charset=utf-8
      
      {"message": "こんにちは、世界！", "status": "成功", "data": [{"name": "田中太郎", "age": 25}]}
      """
    And I am in the response pane
    And the cursor is at column 0
    When I press "j" 3 times
    Then the cursor is at line 3
    When I press "w" 5 times
    Then the terminal state should be valid
    And the cursor position should be valid