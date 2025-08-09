Feature: Unicode and Double-Byte Character Support
  As a user who works with international content
  I want to input and display Unicode characters correctly
  So that I can test APIs with various character encodings

  Background:
    Given the application is started with default settings
    And the request buffer is empty
    And I am in the Request pane

  Scenario: Japanese Hiragana input
    When I enter Insert mode
    And I type "こんにちは"
    Then the screen should not be blank
    And I should see "こんにちは" in the output

  Scenario: Japanese Katakana input  
    When I enter Insert mode
    And I type "カタカナ"
    Then the screen should not be blank
    And I should see "カタカナ" in the output

  Scenario: Japanese Kanji input
    When I enter Insert mode
    And I type "日本語"
    Then the screen should not be blank  
    And I should see "日本語" in the output

  Scenario: Mixed ASCII and Japanese text
    When I enter Insert mode
    And I type "Hello こんにちは World 世界"
    Then the screen should not be blank
    And I should see "Hello こんにちは World 世界" in the output

  Scenario: Chinese characters
    When I enter Insert mode
    And I type "你好世界"
    Then the screen should not be blank
    And I should see "你好世界" in the output

  Scenario: Korean characters
    When I enter Insert mode
    And I type "안녕하세요"
    Then the screen should not be blank
    And I should see "안녕하세요" in the output

  Scenario: Emoji support
    When I enter Insert mode
    And I type "Hello 🌍 World 🚀"
    Then the screen should not be blank
    And I should see "Hello 🌍 World 🚀" in the output

  Scenario: Unicode in HTTP request headers
    When I enter Insert mode
    And I type "POST /api/message"
    And I press Enter
    And I type "Content-Type: application/json"
    And I press Enter
    And I press Enter
    And I type the following JSON:
      """
      {"greeting": "こんにちは", "name": "田中さん"}
      """
    Then the screen should not be blank
    And I should see "こんにちは" in the output
    And I should see "田中さん" in the output

  Scenario: Long lines with double-byte characters
    When I enter Insert mode
    And I type "This is a very long line with Japanese こんにちはこんにちはこんにちは and more English text"
    Then the screen should not be blank
    And I should see "こんにちは" in the output

  Scenario: Backspace with double-byte characters
    Given I am in Insert mode
    And I type "Hello こんにちは"
    When I press Backspace
    When I press Backspace
    When I press Backspace
    Then the screen should not be blank
    And I should see "Hello こん" in the output

  Scenario: Navigation through Unicode text with arrow keys
    Given I am in Insert mode
    And I type "ASCII こんにちは ASCII"
    When I press Escape
    And I am in Normal mode
    And I press "0"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    And I press "l"
    Then the screen should not be blank
    And I should see "こんにちは" in the output

  Scenario: Word navigation with mixed text
    Given I am in Insert mode
    And I type "Hello こんにちは World"
    When I press Escape
    And I am in Normal mode
    And I press "0"
    And I press "w"
    And I press "w"
    Then the screen should not be blank
    And I should see "World" in the output