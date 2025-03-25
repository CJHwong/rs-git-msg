use anyhow::Result;
use std::sync::Arc;

use crate::ai::AiProvider;

pub struct CommitMessageGenerator {
    ai_provider: Arc<Box<dyn AiProvider>>,
}

impl CommitMessageGenerator {
    pub fn new(ai_provider: Box<dyn AiProvider>) -> Self {
        Self {
            ai_provider: Arc::new(ai_provider),
        }
    }

    pub async fn generate(
        &self,
        diff: &str,
        branch_name: &str,
        count: u8,
        additional_instructions: Option<&str>,
    ) -> Result<Vec<String>> {
        let prompt = self.build_prompt(diff, branch_name, count, additional_instructions);
        let response = self.ai_provider.generate_text(&prompt).await?;
        
        // Parse the response into individual commit messages
        let messages = self.parse_response(&response, count);
        Ok(messages)
    }

    fn build_prompt(
        &self,
        diff: &str,
        branch_name: &str,
        count: u8,
        additional_instructions: Option<&str>,
    ) -> String {
        let mut prompt = format!(
            "Generate {} commit message(s) for the following changes.\n\n",
            count
        );
        
        prompt.push_str("Follow the Conventional Commits specification (https://www.conventionalcommits.org/):\n");
        prompt.push_str("- Format: type(scope): subject\n");
        prompt.push_str("- Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert\n");
        prompt.push_str("- Keep the subject concise (under 72 characters)\n");
        prompt.push_str("- Use imperative mood (\"add\" not \"added\")\n\n");
        
        prompt.push_str(&format!("Branch name: {}\n\n", branch_name));
        
        if let Some(instructions) = additional_instructions {
            prompt.push_str(&format!("Additional context: {}\n\n", instructions));
        }
        
        prompt.push_str("Diff:\n```\n");
        prompt.push_str(diff);
        prompt.push_str("\n```\n\n");
        
        prompt.push_str(&format!(
            "Provide exactly {} commit message(s) in the format 'type(scope): subject', numbered if more than one.",
            count
        ));
        
        prompt
    }

    fn parse_response(&self, response: &str, count: u8) -> Vec<String> {
        // Simple parsing logic - could be enhanced for more complex responses
        let lines: Vec<&str> = response.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();
        
        let mut messages = Vec::new();
        
        // If we're expecting multiple messages, look for numbered items
        if count > 1 {
            for line in &lines {
                // Look for lines that start with a number or have conventional commit format
                if (line.starts_with(|c: char| c.is_numeric() && c.is_ascii_digit()) && line.contains(':')) 
                   || line.contains("feat(") || line.contains("fix(") || line.contains("docs(") 
                   || line.contains("style(") || line.contains("refactor(") {
                    // Remove leading numbers if present
                    let message = line.trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ' ' || c == ')');
                    messages.push(message.trim().to_string());
                }
            }
        }
        
        // If parsing failed or expecting just one message, try to find the first conventional commit format
        if messages.is_empty() {
            for line in &lines {
                if line.contains(':') {
                    messages.push(line.trim().to_string());
                    if messages.len() >= count as usize {
                        break;
                    }
                }
            }
        }
        
        // If still empty, just return the first non-empty line
        if messages.is_empty() && !lines.is_empty() {
            messages.push(lines[0].to_string());
        }
        
        // Limit to requested count
        messages.truncate(count as usize);
        
        messages
    }
}
