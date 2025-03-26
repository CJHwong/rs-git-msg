use anyhow::Result;

use crate::ai::AiProvider;

pub struct CommitMessageGenerator<T: AiProvider> {
    ai_provider: T,
}

impl<T: AiProvider> CommitMessageGenerator<T> {
    pub fn new(ai_provider: T) -> Self {
        Self { ai_provider }
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
        prompt.push_str(
            "- Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert\n",
        );
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
        // Return empty vector early if count is 0
        if count == 0 {
            return Vec::new();
        }

        // Simple parsing logic - could be enhanced for more complex responses
        let lines: Vec<&str> = response
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect();

        let mut messages = Vec::new();

        // If we're expecting multiple messages, look for numbered items
        if count > 1 {
            for line in &lines {
                // Look for lines that start with a number or have conventional commit format
                if (line.starts_with(|c: char| c.is_numeric() && c.is_ascii_digit())
                    && line.contains(':'))
                    || line.contains("feat(")
                    || line.contains("fix(")
                    || line.contains("docs(")
                    || line.contains("style(")
                    || line.contains("refactor(")
                {
                    // Remove leading numbers if present
                    let message = line.trim_start_matches(|c: char| {
                        c.is_numeric() || c == '.' || c == ' ' || c == ')'
                    });
                    messages.push(message.trim().to_string());
                }
            }
        }

        // If parsing failed or expecting just one message, try to find the first conventional commit format
        if messages.is_empty() {
            for line in &lines {
                if line.contains(':') {
                    // Also strip number prefixes for single message case
                    let message = line.trim_start_matches(|c: char| {
                        c.is_numeric() || c == '.' || c == ' ' || c == ')'
                    });
                    messages.push(message.trim().to_string());
                    if messages.len() >= count as usize {
                        break;
                    }
                }
            }
        }

        // If still empty, just return the first non-empty line
        if messages.is_empty() && !lines.is_empty() {
            // Also strip number prefixes for fallback case
            let message = lines[0]
                .trim_start_matches(|c: char| c.is_numeric() || c == '.' || c == ' ' || c == ')');
            messages.push(message.trim().to_string());
        }

        // Limit to requested count
        messages.truncate(count as usize);

        messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::mock::MockProvider;

    #[test]
    fn test_build_prompt() {
        let mock_provider = MockProvider::new("test response");
        let generator = CommitMessageGenerator::new(mock_provider);

        let diff = "--- a/file.rs\n+++ b/file.rs\n@@ -1,3 +1,4 @@\n+fn new_function() {}";
        let branch_name = "feature/user-auth";
        let count = 2;
        let instructions = Some("Focus on security improvements");

        let prompt = generator.build_prompt(diff, branch_name, count, instructions);

        // Check that all required components are in the prompt
        assert!(prompt.contains("Generate 2 commit message(s)"));
        assert!(prompt.contains("Branch name: feature/user-auth"));
        assert!(prompt.contains("Additional context: Focus on security improvements"));
        assert!(prompt.contains("fn new_function() {}"));
        assert!(prompt.contains("Follow the Conventional Commits specification"));
    }

    #[test]
    fn test_build_prompt_without_instructions() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        let prompt = generator.build_prompt("some diff", "main", 1, None);

        // Check prompt structure is correct
        assert!(prompt.contains("Generate 1 commit message"));
        assert!(prompt.contains("Branch name: main"));
        assert!(!prompt.contains("Additional context:"));
        assert!(prompt.contains("Provide exactly 1 commit message"));
    }

    #[test]
    fn test_parse_response_single() {
        let mock_provider = MockProvider::new("test response");
        let generator = CommitMessageGenerator::new(mock_provider);

        let response = "feat(auth): implement user authentication";
        let messages = generator.parse_response(response, 1);

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], "feat(auth): implement user authentication");
    }

    #[test]
    fn test_parse_response_multiple() {
        let mock_provider = MockProvider::new("test response");
        let generator = CommitMessageGenerator::new(mock_provider);

        let response =
            "1. feat(auth): implement user authentication\n2. fix(ui): correct button alignment";
        let messages = generator.parse_response(response, 2);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], "feat(auth): implement user authentication");
        assert_eq!(messages[1], "fix(ui): correct button alignment");
    }

    #[test]
    fn test_parse_response_with_extra_content() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        let response = "Here are some commit messages:\n\n1. feat(auth): implement login\n2. fix(api): resolve timeout issue\n\nLet me know if you need more!";
        let messages = generator.parse_response(response, 2);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], "feat(auth): implement login");
        assert_eq!(messages[1], "fix(api): resolve timeout issue");
    }

    #[test]
    fn test_parse_response_more_requested_than_available() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        let response = "feat(core): add new feature";
        let messages = generator.parse_response(response, 3);

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], "feat(core): add new feature");
    }

    #[test]
    fn test_parse_response_no_conventional_format() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        let response = "This is just a simple message without conventional format";
        let messages = generator.parse_response(response, 1);

        assert_eq!(messages.len(), 1);
        assert_eq!(
            messages[0],
            "This is just a simple message without conventional format"
        );
    }

    #[test]
    fn test_parse_response_empty() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        let response = "";
        let messages = generator.parse_response(response, 1);

        assert_eq!(messages.len(), 0);
    }

    #[test]
    fn test_parse_response_with_unconventional_formats() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        let response = "1) First commit\n2) Second: with colon but not conventional\nrefactor(core): proper format";
        let messages = generator.parse_response(response, 3);

        // Should find the proper format one and the one with colon
        assert!(messages.contains(&"Second: with colon but not conventional".to_string()));
        assert!(messages.contains(&"refactor(core): proper format".to_string()));
    }

    #[tokio::test]
    async fn test_generate() {
        let mock_provider = MockProvider::new("feat(test): add new feature");
        let generator = CommitMessageGenerator::new(mock_provider);

        let result = generator.generate("test diff", "main", 1, None).await;

        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], "feat(test): add new feature");
    }

    #[tokio::test]
    async fn test_generate_with_provider_error() {
        let mock_provider = MockProvider::new_with_error("provider error");
        let generator = CommitMessageGenerator::new(mock_provider);

        let result = generator.generate("test diff", "main", 1, None).await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "provider error");
    }

    #[tokio::test]
    async fn test_generate_multiple_messages() {
        let mock_provider = MockProvider::new(
            "1. feat(ui): add login form\n2. feat(api): implement authentication endpoints",
        );
        let generator = CommitMessageGenerator::new(mock_provider);

        let result = generator
            .generate("test diff", "feature/auth", 2, Some("New auth system"))
            .await;

        assert!(result.is_ok());
        let messages = result.unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], "feat(ui): add login form");
        assert_eq!(messages[1], "feat(api): implement authentication endpoints");
    }

    #[tokio::test]
    async fn test_generate_verifies_prompt_contents() {
        let mock_provider = MockProvider::new("test response");
        let provider_calls = mock_provider.calls.clone();
        let generator = CommitMessageGenerator::new(mock_provider);

        let _ = generator
            .generate("test diff", "feature/test", 3, Some("test instructions"))
            .await;

        let calls = provider_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);

        let prompt = &calls[0];
        assert!(prompt.contains("Generate 3 commit message(s)"));
        assert!(prompt.contains("Branch name: feature/test"));
        assert!(prompt.contains("Additional context: test instructions"));
        assert!(prompt.contains("test diff"));
    }

    #[test]
    fn test_additional_context_formatting() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        // Test with various additional instructions
        let with_instruction =
            generator.build_prompt("diff", "branch", 1, Some("Test instruction"));
        assert!(with_instruction.contains("Additional context: Test instruction\n\n"));

        // Test with special characters in instructions
        let with_special_chars =
            generator.build_prompt("diff", "branch", 1, Some("Test: with! special* chars?"));
        assert!(with_special_chars.contains("Additional context: Test: with! special* chars?\n\n"));

        // Test with empty string instruction (should still include the header)
        let with_empty = generator.build_prompt("diff", "branch", 1, Some(""));
        assert!(with_empty.contains("Additional context: \n\n"));
    }

    #[test]
    fn test_message_trimming() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        // Test with leading/trailing spaces
        let response = "  feat(core): trimmed message  ";
        let messages = generator.parse_response(response, 1);
        assert_eq!(messages[0], "feat(core): trimmed message");

        // Test with leading numbers and formatting
        let response = "1. feat(core): first message\n  2)  feat(ui): second message  ";
        let messages = generator.parse_response(response, 2);
        assert_eq!(messages[0], "feat(core): first message");
        assert_eq!(messages[1], "feat(ui): second message");
    }

    #[test]
    fn test_colon_detection() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        // Test with and without colons
        let response = "Line without colon\nfeat(core): with colon\nAnother without";
        let messages = generator.parse_response(response, 1);
        assert_eq!(messages[0], "feat(core): with colon");

        // Test with multiple colon lines
        let response = "First: has colon\nSecond: also has colon";
        let messages = generator.parse_response(response, 2);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], "First: has colon");
        assert_eq!(messages[1], "Second: also has colon");
    }

    #[test]
    fn test_message_truncation() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        // Test truncation
        let response =
            "1. feat(a): first\n2. feat(b): second\n3. feat(c): third\n4. feat(d): fourth";

        // Request fewer than available
        let messages = generator.parse_response(response, 2);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], "feat(a): first");
        assert_eq!(messages[1], "feat(b): second");

        // Request exact number
        let messages = generator.parse_response(response, 4);
        assert_eq!(messages.len(), 4);

        // Request more than available
        let messages = generator.parse_response(response, 6);
        assert_eq!(messages.len(), 4);
    }

    #[tokio::test]
    async fn test_generate_returns_correct_messages() {
        // Test with a multi-line response
        let response =
            "Here are some messages:\n1. feat(a): first message\n2. fix(b): second message";
        let mock_provider = MockProvider::new(response);
        let generator = CommitMessageGenerator::new(mock_provider);

        let result = generator.generate("diff", "branch", 2, None).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "feat(a): first message");
        assert_eq!(result[1], "fix(b): second message");

        // Test with a response that has more messages than requested
        let response = "1. feat(a): first\n2. fix(b): second\n3. docs(c): third";
        let mock_provider = MockProvider::new(response);
        let generator = CommitMessageGenerator::new(mock_provider);

        let result = generator.generate("diff", "branch", 2, None).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "feat(a): first");
        assert_eq!(result[1], "fix(b): second");
    }

    #[test]
    fn test_additional_context_line_specific() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        // Direct test for the specific line that adds additional context
        let instructions = "Very specific test";
        let prompt = generator.build_prompt("diff", "branch", 1, Some(instructions));

        // Verify the exact formatted string that would be created by that line
        let expected_format = format!("Additional context: {}\n\n", instructions);
        assert!(prompt.contains(&expected_format));
    }

    #[test]
    fn test_message_trim_push_specific() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        // This specifically tests the trim and push to string functionality
        let response = "1.  feat(test): with extra spaces  ";
        let messages = generator.parse_response(response, 1);

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], "feat(test): with extra spaces");

        // Test with a single message with number prefix
        let response = "1.     lots   of    spaces    ";
        let messages = generator.parse_response(response, 1);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], "lots   of    spaces");

        // Test with multiple messages, each with colons to ensure they're parsed correctly
        let response = "1. first: message\n2. second: message";
        let messages = generator.parse_response(response, 2);
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], "first: message");
        assert_eq!(messages[1], "second: message");
    }

    #[test]
    fn test_message_truncation_specific() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        // This specifically tests the messages.truncate(count as usize) line
        let response = "1. first: message\n2. second: message\n3. third: message\n4. fourth: message\n5. fifth: message";

        // Test truncation with count > 1 to trigger number stripping logic
        let messages3 = generator.parse_response(response, 3);
        assert_eq!(messages3.len(), 3);

        // Test with more messages than available but still gets properly truncated
        let short_response = "1. one: message\n2. two: message";
        let many_requested = generator.parse_response(short_response, 5);
        assert_eq!(many_requested.len(), 2);

        // Test with count of 0 (edge case)
        let messages0 = generator.parse_response(response, 0);
        assert_eq!(messages0.len(), 0);
    }

    #[test]
    fn test_return_messages_line118() {
        let mock_provider = MockProvider::new("test");
        let generator = CommitMessageGenerator::new(mock_provider);

        // This specifically tests the final return of messages
        let empty_response = "";
        let empty_messages = generator.parse_response(empty_response, 1);
        assert!(empty_messages.is_empty());

        // Test a valid response but with count=0
        let valid_response = "feat: something";
        let zero_messages = generator.parse_response(valid_response, 0);
        assert!(zero_messages.is_empty());

        // Test direct return of constructed messages
        let simple_response = "feat: direct test";
        let messages = generator.parse_response(simple_response, 1);
        assert_eq!(messages, vec!["feat: direct test".to_string()]);
    }
}
