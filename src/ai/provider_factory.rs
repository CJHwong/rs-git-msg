use super::{AiProvider, ollama::OllamaProvider, openai::OpenAIProvider};
use crate::Provider;
use anyhow::{Result, anyhow};

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
        }
        Provider::OpenAI => {
            let api_key = api_key.ok_or_else(|| anyhow!("API key is required for OpenAI"))?;
            let base_url = api_url.unwrap_or("https://api.openai.com/v1");
            Ok(Box::new(OpenAIProvider::new(
                base_url, model, api_key, verbose,
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Provider;

    #[test]
    fn test_create_ollama_provider() {
        let provider = create_provider(
            Provider::Ollama,
            "llama3",
            None,
            Some("http://test-url:11434"),
            false,
        );

        assert!(provider.is_ok());
        // Just check that it doesn't fail - we can't directly inspect the type
        // of the boxed trait object
    }

    #[test]
    fn test_create_ollama_provider_default_url() {
        let provider = create_provider(
            Provider::Ollama,
            "llama3",
            None,
            None, // No URL provided - should use default
            true, // With verbose turned on
        );

        assert!(provider.is_ok());
    }

    #[test]
    fn test_create_openai_provider() {
        let provider = create_provider(
            Provider::OpenAI,
            "gpt-4",
            Some("test-api-key"),
            Some("https://test-openai-url"),
            false,
        );

        assert!(provider.is_ok());
    }

    #[test]
    fn test_create_openai_provider_default_url() {
        let provider = create_provider(
            Provider::OpenAI,
            "gpt-4",
            Some("test-api-key"),
            None, // No URL provided - should use default
            true, // With verbose turned on
        );

        assert!(provider.is_ok());
    }

    #[test]
    fn test_create_openai_provider_missing_key() {
        let provider: Result<Box<dyn AiProvider>> = create_provider(
            Provider::OpenAI,
            "gpt-4",
            None,
            Some("https://test-openai-url"),
            false,
        );

        assert!(provider.is_err());
        assert_eq!(
            provider.unwrap_err().to_string(),
            "API key is required for OpenAI"
        );
    }
}
