use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

use super::AiProvider;

#[derive(Debug)]
pub struct GeminiProvider {
    client: Client,
    base_url: String,
    model: String,
    api_key: String,
    verbose: bool,
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

impl GeminiProvider {
    pub fn new(base_url: &str, model: &str, api_key: &str, verbose: bool) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            api_key: api_key.to_string(),
            verbose,
        }
    }
}

#[async_trait]
impl AiProvider for GeminiProvider {
    async fn generate_text(&self, prompt: &str) -> Result<String> {
        if self.verbose {
            println!("Sending request to Gemini API...");
        }

        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
        };

        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url, self.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                eprintln!("Failed to connect to Gemini API: {e}");
                anyhow!("Connection to Gemini failed: {}", e)
            })?;

        if self.verbose {
            println!("Gemini API response status: {}", response.status());
        }

        let text = response.text().await?;
        if self.verbose {
            println!("Raw response: {text}");
        }

        let json: Value = serde_json::from_str(&text)?;

        if let Some(candidates) = json.get("candidates").and_then(Value::as_array) {
            if let Some(candidate) = candidates.first() {
                if let Some(content) = candidate.get("content") {
                    if let Some(parts) = content.get("parts").and_then(Value::as_array) {
                        if let Some(part) = parts.first() {
                            if let Some(text) = part.get("text").and_then(Value::as_str) {
                                return Ok(text.to_string());
                            }
                        }
                    }
                }
            }
        }

        if let Some(error) = json.get("error") {
            return Err(anyhow!("Gemini API error: {}", error.to_string()));
        }

        Err(anyhow!("Failed to parse Gemini response"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to test the response parsing logic directly
    fn extract_content_from_response(json_str: &str) -> Result<String> {
        let json: Value = serde_json::from_str(json_str)?;

        if let Some(candidates) = json.get("candidates").and_then(Value::as_array) {
            if let Some(candidate) = candidates.first() {
                if let Some(content) = candidate.get("content") {
                    if let Some(parts) = content.get("parts").and_then(Value::as_array) {
                        if let Some(part) = parts.first() {
                            if let Some(text) = part.get("text").and_then(Value::as_str) {
                                return Ok(text.to_string());
                            }
                        }
                    }
                }
            }
        }

        if let Some(error) = json.get("error") {
            return Err(anyhow!("Gemini API error: {}", error.to_string()));
        }

        Err(anyhow!("Failed to parse Gemini response"))
    }

    #[tokio::test]
    async fn test_generate_text_success() {
        let json_str = r#"{
            "candidates": [{
                "content": {
                    "parts": [{ "text": "feat: Implement new feature" }]
                }
            }]
        }"#;

        let result = extract_content_from_response(json_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "feat: Implement new feature");
    }

    #[tokio::test]
    async fn test_generate_text_error() {
        let json_str = r#"{"error": "API key missing"}"#;

        let result = extract_content_from_response(json_str);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Gemini API error: \"API key missing\""
        );
    }
}
