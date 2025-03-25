use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

use super::AiProvider;

#[derive(Debug)]
pub struct OllamaProvider {
    client: Client,
    base_url: String,
    model: String,
    verbose: bool,
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

impl OllamaProvider {
    pub fn new(base_url: &str, model: &str, verbose: bool) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            model: model.to_string(),
            verbose,
        }
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    async fn generate_text(&self, prompt: &str) -> Result<String> {
        if self.verbose {
            println!("Sending request to Ollama API...");
        }

        let request = OllamaRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                eprintln!("Failed to connect to Ollama server: {}", e);
                eprintln!("Make sure Ollama is running on {}", self.base_url);
                anyhow!("Connection to Ollama failed: {}", e)
            })?;

        if self.verbose {
            println!("Ollama API response status: {}", response.status());
        }

        let text = response.text().await?;
        if self.verbose {
            println!("Raw response: {}", text);
        }

        let json: Value = serde_json::from_str(&text)?;

        if let Some(response_text) = json.get("response").and_then(Value::as_str) {
            return Ok(response_text.to_string());
        }

        if let Some(error) = json.get("error").and_then(Value::as_str) {
            return Err(anyhow!("Ollama API error: {}", error));
        }

        if let Some(content) = json
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
        {
            return Ok(content.to_string());
        }

        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to test the response parsing logic directly
    fn extract_content_from_response(json_str: &str) -> Result<String> {
        let json: Value = serde_json::from_str(json_str)?;

        if let Some(response_text) = json.get("response").and_then(Value::as_str) {
            return Ok(response_text.to_string());
        }

        if let Some(error) = json.get("error").and_then(Value::as_str) {
            return Err(anyhow!("Ollama API error: {}", error));
        }

        if let Some(content) = json
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str)
        {
            return Ok(content.to_string());
        }

        Ok(json_str.to_string())
    }

    #[tokio::test]
    async fn test_generate_text_success() {
        let json_str = r#"{"response": "feat(api): implement user authentication"}"#;

        let result = extract_content_from_response(json_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "feat(api): implement user authentication");
    }

    #[tokio::test]
    async fn test_generate_text_error() {
        let json_str = r#"{"error": "Model not found"}"#;

        let result = extract_content_from_response(json_str);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Ollama API error: Model not found"
        );
    }
}
