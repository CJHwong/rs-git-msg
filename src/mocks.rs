#[cfg(test)]
pub mod test_utils {
    use crate::git;
    use anyhow::Result;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    pub struct MockRepository {
        branch_name: String,
        diff: String,
    }

    impl MockRepository {
        pub fn new(branch_name: &str, diff: &str) -> Self {
            Self {
                branch_name: branch_name.to_string(),
                diff: diff.to_string(),
            }
        }
    }

    impl git::GitRepository for MockRepository {
        fn get_branch_name(&self) -> Result<String> {
            Ok(self.branch_name.clone())
        }

        fn get_staged_diff(&self) -> Result<String> {
            Ok(self.diff.clone())
        }
    }

    // Mock AI provider for testing
    pub struct MockAIProvider {
        pub response: Arc<Mutex<Vec<String>>>,
    }

    impl MockAIProvider {
        pub fn new(responses: Vec<String>) -> Self {
            Self {
                response: Arc::new(Mutex::new(responses)),
            }
        }
    }

    impl crate::ai::AIProvider for MockAIProvider {
        async fn generate_commit_message(
            &self,
            _diff: &str,
            _branch_name: &str,
            _count: u8,
            _instructions: Option<&str>,
        ) -> Result<Vec<String>> {
            let mut responses = self.response.lock().unwrap();
            Ok(std::mem::take(&mut *responses))
        }
    }
}
