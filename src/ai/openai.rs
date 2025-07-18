use anyhow::{Result, anyhow};
use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::fmt::Debug;

use super::AiProvider;

#[derive(Debug)]
pub struct OpenAIProvider {
    client: Client,
    base_url: String,
    model: String,
    api_key: String,
    verbose: bool,
}

#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

impl OpenAIProvider {
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
impl AiProvider for OpenAIProvider {
    async fn generate_text(&self, prompt: &str) -> Result<String> {
        if self.verbose {
            println!("Sending request to OpenAI API...");
        }

        let request = OpenAIRequest {
            model: self.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a helpful assistant that generates git commit messages."
                        .to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            temperature: 0.7,
            max_tokens: 1000,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                eprintln!("Failed to connect to OpenAI API: {e}");
                anyhow!("Connection to OpenAI failed: {}", e)
            })?;

        if self.verbose {
            println!("OpenAI API response status: {}", response.status());
        }

        let text = response.text().await?;
        if self.verbose {
            println!("Raw response: {text}");
        }

        let json: Value = serde_json::from_str(&text)?;

        // Extract text from the standard OpenAI response format
        if let Some(content) = json
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
        {
            return Ok(content.to_string());
        }

        // Check for errors
        if let Some(message) = json
            .get("error")
            .and_then(Value::as_object)
            .and_then(|err| err.get("message"))
            .and_then(Value::as_str)
        {
            return Err(anyhow!("OpenAI API error: {}", message));
        }

        Err(anyhow!("Failed to parse OpenAI response"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to test the response parsing logic directly
    fn extract_content_from_response(json_str: &str) -> Result<String> {
        let json: Value = serde_json::from_str(json_str)?;

        // Extract text from the standard OpenAI response format
        if let Some(content) = json
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(Value::as_str)
        {
            return Ok(content.to_string());
        }

        // Check for errors
        if let Some(message) = json
            .get("error")
            .and_then(Value::as_object)
            .and_then(|err| err.get("message"))
            .and_then(Value::as_str)
        {
            return Err(anyhow!("OpenAI API error: {}", message));
        }

        Err(anyhow!("Failed to parse OpenAI response"))
    }

    #[tokio::test]
    async fn test_generate_text_success() {
        let json_str =
            r#"{"choices":[{"message":{"content":"fix(core): resolve null pointer exception"}}]}"#;

        let result = extract_content_from_response(json_str);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "fix(core): resolve null pointer exception");
    }

    #[tokio::test]
    async fn test_generate_text_error() {
        let json_str = r#"{"error":{"message":"Invalid API key"}}"#;

        let result = extract_content_from_response(json_str);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "OpenAI API error: Invalid API key"
        );
    }
}
