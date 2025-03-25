use anyhow::{Result, anyhow};
use crate::Provider;
use super::{AiProvider, ollama::OllamaProvider, openai::OpenAIProvider};

/// Creates an AI provider based on the specified provider type
pub fn create_provider(
    provider_type: Provider,
    model: &str,
    api_key: Option<&str>,
    api_url: Option<&str>,
    verbose: bool,
) -> Result<Box<dyn AiProvider>> {
    match provider_type {
        Provider::Ollama => {
            let base_url = api_url.unwrap_or("http://localhost:11434");
            Ok(Box::new(OllamaProvider::new(base_url, model, verbose)))
        },
        Provider::OpenAI => {
            let api_key = api_key.ok_or_else(|| anyhow!("API key is required for OpenAI"))?;
            let base_url = api_url.unwrap_or("https://api.openai.com/v1");
            Ok(Box::new(OpenAIProvider::new(base_url, model, api_key, verbose)))
        },
    }
}
