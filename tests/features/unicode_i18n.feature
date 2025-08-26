Feature: Unicode and Internationalization Support
  Comprehensive testing for multi-byte characters and international text

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane

  # Basic Unicode input
  Scenario Outline: Input various Unicode character sets
    When I enter Insert mode
    And I type "<text>"
    Then the screen should not be blank
    And I should see "<text>" in the output

    Examples:
      | text                           |
      | こんにちは                     |
      | カタカナ                       |
      | 日本語                         |
      | 你好世界                       |
      | 안녕하세요                     |
      | Hello 🌍 World 🚀              |
      | Hello こんにちは World 世界    |

  # Navigation with multi-byte characters
  Scenario: Navigate through mixed ASCII and Unicode text
    Given I am in Insert mode
    When I type "Hello こんにちは World"
    And I press Escape
    And I press "0"
    When I press "w"
    Then the cursor should be on the first multi-byte character
    When I press "w"
    Then the cursor should be at the start of "World"

  Scenario: Dollar sign navigation with long multi-byte line
    Given I am in Insert mode
    And wrap is off
    When I type "あいうえおかきくけこさしすせそたちつてとなにぬねのはひふへほまみむめもやゆよらりるれろわをん"
    And I press Escape
    And I press "0"
    When I press "$"
    Then the cursor should be at the end of the line
    And horizontal scrolling should be active if needed

  # Editing operations with Unicode
  Scenario: Backspace with multi-byte characters
    Given I am in Insert mode
    When I type "Hello こんにちは"
    And I press Backspace
    And I press Backspace
    And I press Backspace
    Then I should see "Hello こん" in the output

  Scenario: Delete operations in visual mode with Unicode
    Given I am in Insert mode
    When I type "こんにちは World"
    And I press Escape
    And I press "0"
    And I press "v"
    And I press "l"
    And I press "l"
    And I press "d"
    Then I should see "ちは World" in the request pane at line 1

  # JSON with international content
  Scenario: JSON request with Unicode values
    When I enter Insert mode
    And I type "POST /api/message"
    And I press Enter
    And I press Enter
    And I type the following JSON:
      """
      {"greeting": "こんにちは", "name": "田中さん"}
      """
    Then I should see "こんにちは" in the output
    And I should see "田中さん" in the output

  # Edge cases
  Scenario: Very long lines with multi-byte characters
    Given I am in Insert mode
    When I type "This is a very long line with doublebyte こんにちはこんにちはこんにちは and more English text"
    Then the screen should not be blank
    And text wrapping should handle multi-byte characters correctly

  Scenario: Word boundaries with mixed character sets
    Given I am in Insert mode
    When I type "ASCII日本語ASCII한글ASCII"
    And I press Escape
    And I press "0"
    When I press "e"
    Then the cursor should be at the end of "ASCII"
    When I press "e"
    Then the cursor should be at the end of "日本語"