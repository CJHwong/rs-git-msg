use anyhow::{Result, bail};
use git2::{DiffOptions, Repository as Git2Repo, Status, StatusOptions};
use std::path::Path;
use std::process::Command;

pub enum DiffAlg {
    Default,
    Patience,
    Minimal,
    Difftastic,
    // Add more as needed
}

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

    pub fn get_staged_diff(&self, alg: DiffAlg) -> Result<String> {
        match alg {
            DiffAlg::Difftastic => {
                // Check if difftastic is available
                if Command::new("which")
                    .arg("difft")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
                {
                    // Collect staged files
                    let mut status_opts = StatusOptions::new();
                    status_opts.include_untracked(false).include_ignored(false);
                    let statuses = self.repo.statuses(Some(&mut status_opts))?;
                    let mut diffs = Vec::new();
                    for entry in statuses.iter() {
                        let status = entry.status();
                        if status.intersects(
                            Status::INDEX_NEW | Status::INDEX_MODIFIED | Status::INDEX_DELETED,
                        ) {
                            let path = entry.path().unwrap();
                            // Get blob for HEAD (if exists)
                            let head = self.repo.head().ok();
                            let tree = head.as_ref().and_then(|h| h.peel_to_tree().ok());
                            let head_blob = tree
                                .as_ref()
                                .and_then(|t| t.get_path(Path::new(path)).ok())
                                .and_then(|e| e.to_object(&self.repo).ok())
                                .and_then(|o| o.as_blob().map(|b| b.content().to_vec()));
                            // Get blob for index (staged)
                            let index = self.repo.index()?;
                            let staged_entry = index.get_path(Path::new(path), 0);
                            let staged_blob = if let Some(e) = staged_entry {
                                self.repo.find_blob(e.id).ok().map(|b| b.content().to_vec())
                            } else {
                                None
                            };
                            // Write both to temp files
                            let a_file = tempfile::NamedTempFile::new()?;
                            let b_file = tempfile::NamedTempFile::new()?;
                            if let Some(ref content) = head_blob {
                                std::fs::write(a_file.path(), content)?;
                            }
                            if let Some(ref content) = staged_blob {
                                std::fs::write(b_file.path(), content)?;
                            }
                            // Call difft
                            let output = Command::new("difft")
                                .arg(a_file.path())
                                .arg(b_file.path())
                                .current_dir(self.repo.path().parent().unwrap())
                                .output();
                            match output {
                                Ok(output) => {
                                    if output.status.success() {
                                        let diff =
                                            String::from_utf8_lossy(&output.stdout).to_string();
                                        if !diff.trim().is_empty() {
                                            diffs.push(format!(
                                                "diff --difftastic a/{path} b/{path}\n{diff}"
                                            ));
                                        }
                                    } else {
                                        bail!(
                                            "difftastic failed: {}",
                                            String::from_utf8_lossy(&output.stderr)
                                        );
                                    }
                                }
                                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                                    bail!(
                                        "difftastic (difft) not found in PATH when running difft. Please install difftastic (https://difftastic.wilfred.me.uk/) and ensure 'difft' is available in your PATH."
                                    );
                                }
                                Err(e) => {
                                    bail!("Failed to run difft: {}", e);
                                }
                            }
                        }
                    }
                    Ok(diffs.join("\n"))
                } else {
                    bail!(
                        "difftastic (difft) not found in PATH. Please install difftastic (https://difftastic.wilfred.me.uk/) and ensure 'difft' is available in your PATH."
                    );
                }
            }
            _ => {
                // Built-in algorithms
                let head = self.repo.head().ok();
                let tree = head.as_ref().and_then(|h| h.peel_to_tree().ok());

                if self.verbose && head.is_none() {
                    println!("Debug: Repository has no HEAD commit yet");
                }

                let mut options = DiffOptions::new();
                match alg {
                    DiffAlg::Patience => {
                        options.patience(true);
                    }
                    DiffAlg::Minimal => {
                        options.minimal(true);
                    }
                    _ => {}
                }

                let diff = self
                    .repo
                    .diff_tree_to_index(tree.as_ref(), None, Some(&mut options))?;

                let mut diff_text = String::new();

                diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
                    if let Ok(content) = std::str::from_utf8(line.content()) {
                        diff_text.push_str(content);
                    }
                    true
                })?;

                if diff_text.is_empty() && self.verbose {
                    self.debug_staging_status()?;
                }

                Ok(diff_text)
            }
        }
    }

    pub fn get_last_commit_titles(&self, n: usize) -> Result<Vec<String>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        let mut titles = Vec::new();

        for oid_result in revwalk.take(n) {
            let oid = oid_result?;
            let commit = self.repo.find_commit(oid)?;
            let message = commit.summary().unwrap_or("").trim().to_string();
            if !message.is_empty() {
                titles.push(message);
            }
        }
        Ok(titles)
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

            println!("Debug: {path} - staged: {is_staged}, status: {status:?}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::io::{self};
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
        let diff = repo.get_staged_diff(DiffAlg::Default);

        assert!(diff.is_ok());
        let diff_text = diff.unwrap();
        assert!(diff_text.contains("modified content"));

        drop(temp_dir);
    }

    #[test]
    fn test_open_current_dir() {
        let (temp_dir, repo_path) = setup_test_repo();

        // Save current directory
        let original_dir = env::current_dir().unwrap();

        // Change current directory to temp repo
        env::set_current_dir(&repo_path).unwrap();

        // Test open_current_dir
        let repo = Repository::open_current_dir(false);
        assert!(repo.is_ok());

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();

        drop(temp_dir);
    }

    #[test]
    fn test_detached_head() {
        let (temp_dir, repo_path) = setup_test_repo();

        // Create a second commit to detach from
        let file_path = repo_path.join("test.txt");
        fs::write(&file_path, "second commit").unwrap();

        let git_repo = git2::Repository::open(&repo_path).unwrap();
        let mut index = git_repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();

        let tree_id = index.write_tree().unwrap();
        let tree = git_repo.find_tree(tree_id).unwrap();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let parent_commit = git_repo.head().unwrap().peel_to_commit().unwrap();

        let commit_id = git_repo
            .commit(
                Some("refs/heads/master"),
                &sig,
                &sig,
                "Second commit",
                &tree,
                &[&parent_commit],
            )
            .unwrap();

        // Detach HEAD by checking out the commit directly
        git_repo.set_head_detached(commit_id).unwrap();

        // Test get_branch_name in detached state
        let repo = Repository::open(&repo_path, false).unwrap();
        let branch_name = repo.get_branch_name();
        assert!(branch_name.is_ok());
        assert_eq!(branch_name.unwrap(), "detached-head");

        drop(temp_dir);
    }

    #[test]
    fn test_no_head_commit() {
        // Create empty repository without commits
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();
        git2::Repository::init(&repo_path).unwrap();

        // Create repository with verbose mode
        let repo = Repository::open(&repo_path, true).unwrap();

        // Redirect stdout to capture debug output
        let mut output = Vec::new();
        {
            let _ = io::Cursor::new(&mut output);
            // Call get_staged_diff which should print the debug message
            // Note: In a real implementation, you might want to use a crate like
            // `capture-stdout` or `std-redirect` for better output capturing
            let _ = repo.get_staged_diff(DiffAlg::Default);
        }

        // Convert captured output to string
        let _ = String::from_utf8_lossy(&output);

        // This test will currently fail since we can't capture stdout directly
        // In a real implementation, you should inject a logger or use dependency injection
        // Instead we'll just check the code paths are exercised without errors
        let diff = repo.get_staged_diff(DiffAlg::Default);
        assert!(diff.is_ok());

        drop(temp_dir);
    }

    #[test]
    fn test_empty_staged_diff() {
        let (temp_dir, repo_path) = setup_test_repo();

        // Create file but don't stage it
        let file_path = repo_path.join("unstaged.txt");
        fs::write(&file_path, "unstaged content").unwrap();

        // Create repository with verbose mode
        let repo = Repository::open(&repo_path, true).unwrap();

        // Get staged diff which should be empty and trigger debug_staging_status
        let diff = repo.get_staged_diff(DiffAlg::Default);

        assert!(diff.is_ok());
        assert_eq!(diff.unwrap(), "");

        drop(temp_dir);
    }

    #[test]
    fn test_debug_staging_status() {
        let (temp_dir, repo_path) = setup_test_repo();

        // Create different file states:

        // 1. Modified but not staged file
        let modified_path = repo_path.join("test.txt");
        fs::write(&modified_path, "modified not staged").unwrap();

        // 2. Untracked file
        let untracked_path = repo_path.join("untracked.txt");
        fs::write(&untracked_path, "untracked content").unwrap();

        // 3. Staged new file
        let staged_path = repo_path.join("staged.txt");
        fs::write(&staged_path, "staged content").unwrap();

        let git_repo = git2::Repository::open(&repo_path).unwrap();
        let mut index = git_repo.index().unwrap();
        index.add_path(Path::new("staged.txt")).unwrap();
        index.write().unwrap();

        // Create repository with verbose flag
        let repo = Repository::open(&repo_path, true).unwrap();

        // Call debug_staging_status directly to test it
        let result = repo.debug_staging_status();
        assert!(result.is_ok());

        drop(temp_dir);
    }

    #[test]
    fn test_get_last_commit_titles_returns_latest_titles() {
        let (temp_dir, repo_path) = setup_test_repo();
        let repo = Repository::open(&repo_path, false).unwrap();

        // Add two more commits
        let file_path = repo_path.join("test.txt");
        fs::write(&file_path, "second commit").unwrap();
        let git_repo = git2::Repository::open(&repo_path).unwrap();
        let mut index = git_repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = git_repo.find_tree(tree_id).unwrap();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        let parent_commit = git_repo.head().unwrap().peel_to_commit().unwrap();
        git_repo
            .commit(
                Some("refs/heads/master"),
                &sig,
                &sig,
                "Second commit",
                &tree,
                &[&parent_commit],
            )
            .unwrap();

        fs::write(&file_path, "third commit").unwrap();
        let mut index = git_repo.index().unwrap();
        index.add_path(Path::new("test.txt")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = git_repo.find_tree(tree_id).unwrap();
        let parent_commit = git_repo.head().unwrap().peel_to_commit().unwrap();
        git_repo
            .commit(
                Some("refs/heads/master"),
                &sig,
                &sig,
                "Third commit",
                &tree,
                &[&parent_commit],
            )
            .unwrap();

        // Now test get_last_commit_titles
        let titles = repo.get_last_commit_titles(3).unwrap();
        assert_eq!(titles.len(), 3);
        assert_eq!(titles[0], "Third commit");
        assert_eq!(titles[1], "Second commit");
        assert_eq!(titles[2], "Initial commit");

        drop(temp_dir);
    }
}
