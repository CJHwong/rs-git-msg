use async_trait::async_trait;
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Serialize};
use serde_json::Value;

use super::AiProvider;

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

        let response = self.client
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
        
        if let Some(content) = json.get("message")
            .and_then(|m| m.get("content"))
            .and_then(Value::as_str) {
            return Ok(content.to_string());
        }
        
        Ok(text)
    }
}
