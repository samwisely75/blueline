Feature: Unicode and Double-Byte Character Support
  As a user who works with international content
  I want to input and display Unicode characters correctly
  So that I can test APIs with various character encodings

  Background:
    Given blueline is launched in a terminal
    And I am in the request pane

  Scenario: Japanese Hiragana input
    When I enter insert mode
    And I type "こんにちは" (Hello in Japanese)
    Then the screen should not be blank
    And I should see "こんにちは" displayed correctly
    And the cursor position should account for double-byte width
    And the line numbers should align properly

  Scenario: Japanese Katakana input  
    When I enter insert mode
    And I type "カタカナ" (Katakana)
    Then the screen should not be blank
    And I should see "カタカナ" displayed correctly
    And the text should not overflow the pane boundaries

  Scenario: Japanese Kanji input
    When I enter insert mode
    And I type "日本語" (Japanese language)
    Then the screen should not be blank  
    And I should see "日本語" displayed correctly
    And character width calculation should be accurate

  Scenario: Mixed ASCII and Japanese text
    When I enter insert mode
    And I type "Hello こんにちは World 世界"
    Then the screen should not be blank
    And all characters should be visible
    And ASCII and Japanese characters should align properly
    And the cursor should move correctly through mixed text

  Scenario: Chinese characters
    When I enter insert mode
    And I type "你好世界" (Hello World in Chinese)
    Then the screen should not be blank
    And Chinese characters should display correctly
    And character boundaries should be respected

  Scenario: Korean characters
    When I enter insert mode
    And I type "안녕하세요" (Hello in Korean)
    Then the screen should not be blank
    And Korean characters should display correctly

  Scenario: Emoji support
    When I enter insert mode
    And I type "Hello 🌍 World 🚀"
    Then the screen should not be blank
    And emojis should be displayed if supported
    And text layout should not be corrupted

  Scenario: Unicode in HTTP requests
    Given I type a request with Unicode content:
      """
      POST /api/message HTTP/1.1
      Host: example.com
      Content-Type: application/json

      {"greeting": "こんにちは", "name": "田中さん"}
      """
    When I execute the request
    Then the screen should not be blank
    And Unicode characters should be preserved in the request
    And the response should handle Unicode correctly

  Scenario: Long lines with double-byte characters
    When I enter insert mode
    And I type a long line with mixed content:
    """
    This is a very long line with Japanese こんにちはこんにちはこんにちは and more English text to test wrapping behavior with double-byte characters
    """
    Then the screen should not be blank
    And text should wrap correctly at word boundaries
    And double-byte characters should not be split incorrectly
    And line numbers should remain aligned

  Scenario: Backspace with double-byte characters
    Given I am in insert mode
    And I have typed "Hello こんにちは"
    When I press backspace 3 times
    Then the screen should not be blank
    And I should see "Hello こん"
    And character deletion should respect Unicode boundaries

  Scenario: Navigation through Unicode text
    Given I have text "ASCII こんにちは ASCII" 
    And the cursor is at the beginning
    When I press "l" to move right through the text
    Then the cursor should move correctly through mixed characters
    And the screen should not be blank
    And cursor position should account for character widths