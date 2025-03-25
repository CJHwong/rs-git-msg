use async_trait::async_trait;
use anyhow::Result;

pub mod ollama;
pub mod openai;
pub mod provider_factory;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn generate_text(&self, prompt: &str) -> Result<String>;
}

#[async_trait]
impl AiProvider for Box<dyn AiProvider> {
    async fn generate_text(&self, prompt: &str) -> Result<String> {
        (**self).generate_text(prompt).await
    }
}
