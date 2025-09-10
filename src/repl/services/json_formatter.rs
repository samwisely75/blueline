//! # JSON Formatting Service
//!
//! Provides JSON formatting functionality for HTTP responses.
//! Formats JSON content using serde_json::to_string_pretty.

/// JSON formatting service for pretty-printing JSON content
#[derive(Debug, Default)]
pub struct JsonFormatterService;

impl JsonFormatterService {
    /// Create a new JSON formatter service
    pub fn new() -> Self {
        Self
    }

    /// Format JSON content with pretty printing
    ///
    /// This function attempts to parse and reformat JSON content.
    /// If parsing fails, it returns the original content unchanged.
    ///
    /// # Arguments
    ///
    /// * `content` - The raw JSON content to format
    ///
    /// # Returns
    ///
    /// Formatted JSON string if parsing is successful, otherwise the original content
    pub fn format_json(&self, content: &str) -> String {
        tracing::debug!(
            "JsonFormatterService::format_json called with content length: {}",
            content.len()
        );
        tracing::debug!(
            "Content preview: {}",
            &content[..std::cmp::min(200, content.len())]
        );

        // Attempt to parse and reformat JSON
        match serde_json::from_str::<serde_json::Value>(content) {
            Ok(json_value) => {
                tracing::debug!("JSON parsing successful, attempting to format...");
                match serde_json::to_string_pretty(&json_value) {
                    Ok(formatted) => {
                        tracing::info!("Successfully formatted JSON content - original: {} chars, formatted: {} chars", content.len(), formatted.len());
                        tracing::debug!(
                            "Formatted preview: {}",
                            &formatted[..std::cmp::min(200, formatted.len())]
                        );
                        formatted
                    }
                    Err(e) => {
                        tracing::warn!("Failed to format JSON to pretty string: {}", e);
                        content.to_string()
                    }
                }
            }
            Err(e) => {
                tracing::debug!("Content is not valid JSON, returning original: {}", e);
                content.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_formatter_service_should_create() {
        let _service = JsonFormatterService::new();
        // Service creation should not panic
    }

    #[test]
    fn format_json_should_handle_invalid_json() {
        let service = JsonFormatterService::new();
        let invalid_content = r#"{"invalid":"json",}"#; // Trailing comma makes it invalid

        let result = service.format_json(invalid_content);

        // Should return original content when JSON is invalid
        assert_eq!(result, invalid_content);
    }

    #[test]
    fn format_json_should_handle_empty_content() {
        let service = JsonFormatterService::new();
        let content = "";

        let result = service.format_json(content);

        // Empty content should return as-is since it's not valid JSON
        assert_eq!(result, content);
    }

    #[test]
    fn format_json_should_format_valid_json() {
        let service = JsonFormatterService::new();
        let content = r#"{"key":"value","nested":{"data":123}}"#;

        let result = service.format_json(content);

        // Should be formatted (pretty-printed)
        assert!(result.contains("  "));
        assert!(result.contains("\n"));
        assert!(result.contains("key"));
        assert!(result.contains("value"));

        // Verify it's still valid JSON by parsing it again
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["key"], "value");
        assert_eq!(parsed["nested"]["data"], 123);
    }

    #[test]
    fn format_json_should_handle_complex_nested_json() {
        let service = JsonFormatterService::new();
        let content = r#"{"users":[{"id":1,"name":"John","active":true},{"id":2,"name":"Jane","active":false}],"meta":{"total":2,"page":1}}"#;

        let result = service.format_json(content);

        // Should be properly formatted with indentation
        assert!(result.contains("  "));
        assert!(result.contains("\n"));
        assert!(result.contains("users"));
        assert!(result.contains("John"));
        assert!(result.contains("meta"));

        // Verify it's still valid JSON by parsing it again
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["users"][0]["name"], "John");
        assert_eq!(parsed["meta"]["total"], 2);
    }
}
