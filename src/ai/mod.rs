use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;

pub mod gemini;
pub mod ollama;
pub mod openai;
pub mod provider_factory;

#[cfg(test)]
pub mod mock;

#[async_trait]
pub trait AiProvider: Send + Sync + Debug {
    async fn generate_text(&self, prompt: &str) -> Result<String>;
}

#[async_trait]
impl AiProvider for Box<dyn AiProvider> {
    async fn generate_text(&self, prompt: &str) -> Result<String> {
        (**self).generate_text(prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock::MockProvider;

    #[tokio::test]
    async fn test_box_ai_provider_trait() {
        // Create a mock provider
        let mock_provider = MockProvider::new("test response");
        let boxed: Box<dyn AiProvider> = Box::new(mock_provider);

        // Test that the trait is implemented correctly
        let result = boxed.generate_text("test prompt").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test response");
    }

    #[tokio::test]
    async fn test_box_ai_provider_tracks_calls() {
        let mock_provider = MockProvider::new("test response");
        let provider_calls = mock_provider.calls.clone();
        let boxed: Box<dyn AiProvider> = Box::new(mock_provider);

        // Test that calls are tracked properly
        let _ = boxed.generate_text("test prompt 1").await;
        let _ = boxed.generate_text("test prompt 2").await;

        let calls = provider_calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0], "test prompt 1");
        assert_eq!(calls[1], "test prompt 2");
    }

    #[tokio::test]
    async fn test_box_ai_provider_error_propagation() {
        let provider = Box::new(MockProvider::new_with_error("Test error"));

        let result = provider.generate_text("test").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Test error");
    }
}
