use anyhow::Result;
use git2::{DiffOptions, Repository as Git2Repo, Status, StatusOptions};
use std::path::Path;

pub struct Repository {
    repo: Git2Repo,
    verbose: bool,
}

impl Repository {
    pub fn open_current_dir(verbose: bool) -> Result<Self> {
        let path = std::env::current_dir()?;
        Self::open(&path, verbose)
    }

    pub fn open(path: &Path, verbose: bool) -> Result<Self> {
        let repo = Git2Repo::open(path)?;
        Ok(Self { repo, verbose })
    }

    pub fn get_branch_name(&self) -> Result<String> {
        let head = self.repo.head()?;
        if head.is_branch() {
            head.shorthand()
                .map(String::from)
                .ok_or_else(|| anyhow::anyhow!("Failed to get branch name"))
        } else {
            Ok("detached-head".to_string())
        }
    }

    pub fn get_staged_diff(&self) -> Result<String> {
        let head = self.repo.head().ok();
        // Use as_ref() to borrow the Option's contents rather than taking ownership
        let tree = head.as_ref().and_then(|h| h.peel_to_tree().ok());

        if self.verbose && head.is_none() {
            println!("Debug: Repository has no HEAD commit yet");
        }

        let mut options = DiffOptions::new();
        let diff = self
            .repo
            .diff_tree_to_index(tree.as_ref(), None, Some(&mut options))?;

        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_, _, line| {
            if matches!(line.origin(), 'H' | '+' | '-') {
                if let Ok(content) = std::str::from_utf8(line.content()) {
                    diff_text.push_str(content);
                }
            }
            true
        })?;

        if diff_text.is_empty() && self.verbose {
            self.debug_staging_status()?;
        }

        Ok(diff_text)
    }

    fn debug_staging_status(&self) -> Result<()> {
        println!("Debug: No changes detected in staging area. Checking repository status:");

        let mut status_opts = StatusOptions::new();
        status_opts.include_untracked(true);

        let statuses = self.repo.statuses(Some(&mut status_opts))?;

        if statuses.is_empty() {
            println!("Debug: No changes in the repository");
            return Ok(());
        }

        println!("Debug: Found {} changed files:", statuses.len());
        for entry in statuses.iter() {
            let status = entry.status();
            let is_staged = status
                .intersects(Status::INDEX_NEW | Status::INDEX_MODIFIED | Status::INDEX_DELETED);
            let path = String::from_utf8_lossy(entry.path_bytes());

            println!(
                "Debug: {} - staged: {}, status: {:?}",
                path, is_staged, status
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup_test_repo() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize a git repository
        let repo = git2::Repository::init(&repo_path).unwrap();

        // Create an initial commit on master branch
        let file_path = repo_path.join("test.txt");
        fs::write(&file_path, "initial content").unwrap();

        let mut index = repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();

        // Create master branch with initial commit
        repo.commit(
            Some("refs/heads/master"),
            &sig,
            &sig,
            "Initial commit",
            &tree,
            &[],
        )
        .unwrap();

        // Set HEAD to point to master branch
        repo.set_head("refs/heads/master").unwrap();

        (temp_dir, repo_path)
    }

    #[test]
    fn test_open_repository() {
        let (temp_dir, repo_path) = setup_test_repo();
        let repo = Repository::open(&repo_path, false);

        assert!(repo.is_ok());

        // Keep temp_dir alive until the end of the test
        drop(temp_dir);
    }

    #[test]
    fn test_get_branch_name() {
        let (temp_dir, repo_path) = setup_test_repo();
        let repo = Repository::open(&repo_path, false).unwrap();

        let branch_name = repo.get_branch_name();
        assert!(branch_name.is_ok());
        assert_eq!(branch_name.unwrap(), "master");

        drop(temp_dir);
    }

    #[test]
    fn test_get_staged_diff() {
        let (temp_dir, repo_path) = setup_test_repo();

        // Modify file and stage changes
        let file_path = repo_path.join("test.txt");
        fs::write(&file_path, "modified content").unwrap();

        let git_repo = git2::Repository::open(&repo_path).unwrap();
        let mut index = git_repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let repo = Repository::open(&repo_path, false).unwrap();
        let diff = repo.get_staged_diff();

        assert!(diff.is_ok());
        let diff_text = diff.unwrap();
        assert!(diff_text.contains("modified content"));

        drop(temp_dir);
    }
}
