use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use super::AiProvider;

/// A mock AI provider for testing purposes
#[derive(Debug)]
pub struct MockProvider {
    response: String,
    pub calls: Arc<Mutex<Vec<String>>>,
}

impl MockProvider {
    pub fn new(response: impl Into<String>) -> Self {
        Self {
            response: response.into(),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    // Add new method to create error-returning mock
    pub fn new_with_error(error_message: impl Into<String>) -> Self {
        Self {
            response: format!("ERROR:{}", error_message.into()),
            calls: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl AiProvider for MockProvider {
    async fn generate_text(&self, prompt: &str) -> Result<String> {
        // Record the prompt that was passed
        self.calls.lock().unwrap().push(prompt.to_string());

        // Check if this is an error mock
        if self.response.starts_with("ERROR:") {
            return Err(anyhow::anyhow!("{}", &self.response[6..]));
        }

        // Return the predefined response
        Ok(self.response.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockProvider::new("mock response");

        let result = provider.generate_text("test prompt").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mock response");

        let calls = provider.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], "test prompt");
    }

    #[tokio::test]
    async fn test_mock_provider_multiple_calls() {
        let provider = MockProvider::new("mock response");

        let _ = provider.generate_text("prompt 1").await;
        let _ = provider.generate_text("prompt 2").await;
        let result = provider.generate_text("prompt 3").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "mock response");

        let calls = provider.get_calls();
        assert_eq!(calls.len(), 3);
        assert_eq!(calls[0], "prompt 1");
        assert_eq!(calls[1], "prompt 2");
        assert_eq!(calls[2], "prompt 3");
    }

    #[tokio::test]
    async fn test_mock_provider_with_error() {
        let provider = MockProvider::new_with_error("test error message");

        let result = provider.generate_text("test prompt").await;
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "test error message");

        let calls = provider.get_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], "test prompt");
    }

    #[test]
    fn test_get_calls_with_concurrent_access() {
        let provider = MockProvider::new("response");
        let calls = provider.calls.clone();

        // Manually add some calls
        calls.lock().unwrap().push("call 1".to_string());
        calls.lock().unwrap().push("call 2".to_string());

        // Verify get_calls returns the correct data
        let result = provider.get_calls();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "call 1");
        assert_eq!(result[1], "call 2");
    }
}
